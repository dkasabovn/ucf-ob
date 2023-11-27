use std::os::unix::net::UnixStream;
use std::sync::Mutex;

pub struct Client {
    inner: Mutex<Inner>,
}

struct Inner {
    stream: UnixStream
}

impl Inner {
    pub fn new(addr: &'static str) -> Option<Self> {
        match UnixStream::connect(addr) {
            Ok(stream) => Some(Inner {
                stream
            }),
            Err(_) => None
        }
    }
}

impl Client {
    pub fn new(addr: &'static str) -> Option<Self> {
        match Inner::new(addr) {
            Some(client) => Some(Client {
                inner: Mutex::new(client),
            }),
            None => None
        }
    }

    pub fn add_order(&mut self, ) {
        let stream = &self.inner.lock().unwrap().stream;
    }
    pub fn cancel_order(&mut self) {
    }
    pub fn reduce_order(&mut self) {
    }
}
