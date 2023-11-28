use crate::comm::stream::InnerStream;
use crate::comm::repo::InnerRepo;
use crate::comm::domain::*;
use crate::comm::urcp::*;

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
        let req_balance: i32 = (qty * math_price).try_into().expect("failed req balance math");

        if user.balance < req_balance  {
            return None;
        }

        let ret = stream.add_order(qty, price, book_id).unwrap();

        let mut add_response = None;

        for result in ret.iter() {
            println!("{:?}", result);
            unsafe {
                match result {
                    OBResponseWrapper { resp: OBResponse { execute: resp }, typ: OBRespType::EXECUTE } => {
                        repo.create_contract(user.id, resp.executed_oid, resp.qty).unwrap();
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
    pub fn cancel_order(&self, user: &User, oid: usize, book_id: u16) -> Option<()> {
        let mut repo = self.inner.repo.lock().unwrap();
        let mut stream = self.inner.stream.lock().unwrap();

        match stream.cancel_order(oid, book_id) {
            Ok(_) => {
                repo.delete_order(oid).unwrap();
                Some(())
            },
            _ => None
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
}
