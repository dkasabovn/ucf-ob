use crate::comm::stream::InnerStream;
use crate::comm::repo::InnerRepo;
use crate::comm::domain::*;
use crate::comm::urcp::*;

use std::sync::MutexGuard;
use std::sync::Mutex;
use std::sync::Arc;
use std::io;

pub struct InnerClient {
    stream: Mutex<InnerStream>,
    repo: Mutex<InnerRepo>,
}

#[derive(Clone)]
pub struct Client {
    inner: Arc<InnerClient>,
}


// !!! Lock repo first then stream
impl Client {
    pub fn new(addr: &'static str) -> io::Result<Self> {
        let stream = InnerStream::new(addr)?;
        let repo =  InnerRepo::new()?;

        let inner_client = InnerClient{
            stream: Mutex::new(stream),
            repo: Mutex::new(repo),
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
    pub fn add_order(&self, user: &User, price: i8, qty: u64, book_id: u16) -> Option<()> {
        let mut repo = self.inner.repo.lock().unwrap();
        let mut stream = self.inner.stream.lock().unwrap();

        if user.balance < qty as i32 {
            return None;
        }

        let ret = stream.add_order(qty, price, book_id).unwrap();

        for result in ret.iter() {
            unsafe {
                match result {
                    OBResponseWrapper { resp: OBResponse { execute: resp }, typ: OBRespType::EXECUTE } => {
                        let x_resp = repo.create_contract(user.id, resp.executed_oid, resp.qty);
                        // TODO(nw): blocking send that order was executed to that user id to a tokio
                        // broadcast
                    },
                    OBResponseWrapper { resp: OBResponse { add: resp }, typ: OBRespType::ADD } => {
                        let add_resp = repo.add_order_to_user(resp.oid, book_id, price, resp.qty, user.id);
                        // TODO(nw): same shit
                    },
                    _ => unreachable!()
                }
            }
        }

        Some(())
    }
}
