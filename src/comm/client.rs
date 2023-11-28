use crate::comm::stream::InnerStream;
use crate::comm::repo::InnerRepo;

use std::sync::Mutex;
use std::sync::Arc;
use std::io;

pub struct InnerClient {
    stream: Mutex<InnerStream>,
    repo: Mutex<InnerRepo>,
}

pub struct Client {
    inner: Arc<InnerClient>,
}

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
}
