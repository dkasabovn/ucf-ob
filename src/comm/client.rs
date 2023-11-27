use crate::comm::urcp::*;

use std::os::unix::net::UnixStream;
use std::sync::Mutex;
use std::io::Result;

pub struct Inner {
    stream: UnixStream
}

impl Inner {
    pub fn new(addr: &'static str) -> Result<Self> {
        let stream = UnixStream::connect(addr)?;
        stream.set_nonblocking(false)?;
        // TODO: look at these maybe set write_timeout and read_timeout
        stream.set_read_timeout(None)?;
        stream.set_write_timeout(None)?;
        Ok(Self{
            stream
        })
    }
    pub fn add_order(&mut self, qty: u64, price: i8, ob_id: u16) -> Result<Vec<OBResponseWrapper>> {
        write_request(&mut self.stream, &OBReqType::ADD, &OBRequest{ add: AddRequest::new(qty, price, ob_id) })?;
        read_response_vec(&mut self.stream)
    }
    pub fn cancel_order(&mut self, oid: usize, ob_id: u16) -> Result<OBResponseWrapper> {
        write_request(&mut self.stream, &OBReqType::CANCEL, &OBRequest{ cancel: CancelRequest::new(oid, ob_id) })?;
        read_response(&mut self.stream)
    }
    pub fn reduce_order(&mut self, oid: usize, qty: u64, ob_id: u16) -> Result<OBResponseWrapper> {
        write_request(&mut self.stream, &OBReqType::REDUCE, &OBRequest{ reduce: ReduceRequest::new(oid, qty, ob_id) })?;
        read_response(&mut self.stream)
    }
}

