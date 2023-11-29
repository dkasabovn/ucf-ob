use crate::comm::stream::InnerStream;
use crate::comm::repo::InnerRepo;
use crate::comm::domain::*;
use crate::comm::urcp::*;

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::Arc;
use std::io;

use tokio::sync::broadcast::Sender;
use serde_json::to_string;

pub struct InnerClient {
    stream: Mutex<InnerStream>,
    repo: Mutex<InnerRepo>,
    sender: Sender<String>,
}

pub struct Client {
    inner: Arc<InnerClient>,
}


impl Clone for Client {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}


// !!! Lock repo first then stream
impl Client {
    pub fn new(addr: &'static str, sender: Sender<String>) -> io::Result<Self> {
        let stream = InnerStream::new(addr)?;
        let repo =  InnerRepo::new()?;

        let inner_client = InnerClient{
            stream: Mutex::new(stream),
            repo: Mutex::new(repo),
            sender: sender,
        };

        Ok(Client {
            inner: Arc::new(inner_client),
        })
    }
    pub fn get_user(&self, sub: String) -> Option<User> {
        let mut repo = self.inner.repo.lock().unwrap();
        match repo.get_user(sub) {
            Ok(usr) => Some(usr),
            _ => None
        }
    }
    pub fn create_user(&self, sub: String) -> Option<()> {
        let mut repo = self.inner.repo.lock().unwrap();
        match repo.create_user(sub) {
            Ok(_) => Some(()),
            _ => None
        }
    }
    pub fn add_order(&self, user: &User, price: i8, qty: u64, book_id: u16) -> Option<AddResponse> {
        let mut repo = self.inner.repo.lock().unwrap();
        let mut stream = self.inner.stream.lock().unwrap();
    
        let math_price: u64 = price.abs() as u64;
        let req_balance: i32 = match (qty * math_price).try_into() {
            Ok(bal) => bal,
            Err(_) => return None
        };

        if user.balance < req_balance  {
            return None;
        }

        println!("Adding order P: {} Q: {} B: {}", price, qty, book_id);

        let ret = match stream.add_order(qty, price, book_id) {
            Err(e) => {
                println!("{}", e);
                return None;
            },
            Ok(data) => data,
        };

        let _ = repo.modify_user_balance(user.id, -req_balance);

        let mut add_response = None;

        for result in ret.iter() {
            println!("In Client: {:?}", result);
            unsafe {
                match result {
                    OBResponseWrapper { resp: OBResponse { execute: resp }, typ: OBRespType::EXECUTE } => {
                        repo.create_contract(user.id, resp.executed_oid, resp.qty, book_id).unwrap();
                        let execute_packet = ApiExecuteResponse {
                            typ: String::from("execute"),
                            data: ApiExecuteInner {
                                oid: resp.executed_oid,
                                qty: resp.qty,
                            }
                        };
                        if let Ok(json) = to_string(&execute_packet) {
                            match self.inner.sender.send(json) {
                                Err(e) => println!("ERROR BCAST: {}", e),
                                _ => (),
                            }
                        }
                        
                    },
                    OBResponseWrapper { resp: OBResponse { add: resp }, typ: OBRespType::ADD } => {
                        repo.add_order_to_user(resp.oid, book_id, price, resp.qty, user.id).unwrap();
                        add_response = Some(resp.clone());
                    },
                    _ => unreachable!()
                }
            }
        }

        match add_response {
            Some(e) => Some(e),
            None => Some(AddResponse{
                qty: 0,
                oid: 0,
            })
        }
    }
    pub fn reduce_order(&self, user: &User, oid: usize, qty: u64, book_id: u16) -> Option<()> {
        let mut repo = self.inner.repo.lock().unwrap();
        let mut stream = self.inner.stream.lock().unwrap();

        // check that we can actually perform this operation

        match stream.reduce_order(oid, qty, book_id) {
            Ok(_) => {
                repo.modify_user_balance(user.id, -(qty as i32)).unwrap();
                Some(())
            },
            _ => None
        }
    }
    pub fn cancel_order(&self, _user: &User, oid: usize, book_id: u16) -> Option<()> {
        println!("cancel start");
        let mut repo = self.inner.repo.lock().unwrap();
        let mut stream = self.inner.stream.lock().unwrap();

        match repo.delete_order(oid) {
            Ok(cnt) => if cnt <= 0 {
                return None
            },
            _ => ()
        };

        match stream.cancel_order(oid, book_id) {
            Ok(_) => {
                println!("cancel end");
                Some(())
            },
            Err(e) => {
                println!("CANCEL: {}", e);
                None
            }
        }
    }
    pub fn flush_exchange(&self, top: bool, right: bool) -> Option<()> {
        let mut repo = self.inner.repo.lock().unwrap();
        let mut stream = self.inner.stream.lock().unwrap();

        let mut map: BTreeMap<i32, i32> = BTreeMap::new();

        let orders = repo.get_all_orders();

        if let Ok(orders) = orders {
            for order in orders.iter() {
                let rp = if order.price < 0 {
                    -order.price
                } else {
                    100 - order.price
                };
                let return_balance = rp * order.qty;
                match map.get_mut(&order.user_fk) {
                    Some(v) => {
                        *v += return_balance;
                    },
                    None => {
                        map.insert(order.user_fk, return_balance);
                    }
                }
            }
        }

        if let Ok(contracts) = repo.get_contracts() {
            for contract in contracts.iter() {
                if contract.book_id == 0 {
                    let add_user_id = if right {
                        contract.yes_holder
                    } else {
                        contract.no_holder
                    };

                    match map.get_mut(&add_user_id) {
                        Some(v) => {
                            *v += contract.qty * 100;
                        },
                        None => {
                            map.insert(add_user_id, contract.qty * 100);
                        }
                    }
                }
            }
        }

        for (uid, bonus) in map.iter() {
            let _ = repo.modify_user_balance(*uid, *bonus);
        }

        if let Ok(json) = to_string(&ApiResultResponse{
            top,
            right
        }) {
            match self.inner.sender.send(json) {
                Err(e) => println!("ERROR BCAST: {}", e),
                _ => (),
            }
        }

        let _ = repo.drop_orders();

        // todo(nw) after done call stream.flush
        let _ = stream.flush_book(0);
        let _ = stream.flush_book(1);

        None 
    }
    pub fn get_contracts_for_user(&self, uid: i32) -> Option<Vec<Contract>> {
        let mut repo = self.inner.repo.lock().unwrap();
        match repo.get_contracts_for_user(uid) {
            Err(_) => None,
            Ok(data) => Some(data),
        }
    }
    pub fn get_ob_levels(&self) -> Vec<std::collections::BTreeMap<i8, u64>> {
        let stream = self.inner.stream.lock().unwrap();
        stream.get_price_levels()
    }

    pub fn get_orders(&self, user: &User) -> Option<Vec<UserOrder>> {
        let mut repo = self.inner.repo.lock().unwrap();

        match repo.get_orders(user.id) {
            Ok(orders) => Some(orders),
            _ => None
        }
    }
    pub fn get_leaderboard(&self) -> Option<Vec<User>> {
        let mut repo = self.inner.repo.lock().unwrap();
        match repo.get_order_leaderboard() {
            Ok(data) => Some(data),
            _ => None
        }
    }
}
