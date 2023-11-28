use crate::comm::urcp::*;

use std::collections::BTreeMap;
use std::os::unix::net::UnixStream;
use std::io::Result;

pub struct InnerStream {
    stream: UnixStream,
    prices: BTreeMap<i8, u64>,
}

impl InnerStream {
    pub fn new(addr: &'static str) -> Result<Self> {
        let stream = UnixStream::connect(addr)?;
        stream.set_nonblocking(false)?;
        // TODO: look at these maybe set write_timeout and read_timeout
        stream.set_read_timeout(None)?;
        stream.set_write_timeout(None)?;
        Ok(Self{
            stream,
            prices: BTreeMap::new(),
        })
    }
    pub fn add_order(&mut self, qty: u64, price: i8, ob_id: u16) -> Result<Vec<OBResponseWrapper>> {
        write_request(&mut self.stream, &OBReqType::ADD, &OBRequest{ add: AddRequest::new(qty, price, ob_id) })?;
        let mut responses = read_response_vec(&mut self.stream)?;
        for response in responses.iter() {
            println!("{:?}", response);
        }
        responses.retain(|x| {
            if matches!(x.typ, OBRespType::PRICE) {
                unsafe { self.handle_price_level(x.resp.price); }
                false
            } else {
                true
            }
        });
        Ok(responses)
    }
    pub fn cancel_order(&mut self, oid: usize, ob_id: u16) -> Result<()> {
        write_request(&mut self.stream, &OBReqType::CANCEL, &OBRequest{ cancel: CancelRequest::new(oid, ob_id) })?;
        let price_level = read_response(&mut self.stream)?;
        assert!(matches!(price_level.typ, OBRespType::PRICE));
        unsafe {
            self.handle_price_level(price_level.resp.price);
        }
        Ok(())
    }
    pub fn reduce_order(&mut self, oid: usize, qty: u64, ob_id: u16) -> Result<()> {
        write_request(&mut self.stream, &OBReqType::REDUCE, &OBRequest{ reduce: ReduceRequest::new(oid, qty, ob_id) })?;
        let price_level = read_response(&mut self.stream)?;
        assert!(matches!(price_level.typ, OBRespType::PRICE));
        unsafe {
            self.handle_price_level(price_level.resp.price);
        }
        Ok(())
    }
    pub fn handle_price_level(&mut self, plu: PriceLevelResponse) {
        match self.prices.get_mut(&plu.price) {
            None => {
                self.prices.insert(plu.price, plu.delta as u64);
            },
            Some(entry) => {
                if plu.delta < 0 {
                    *entry -= plu.delta.abs() as u64
                } else {
                    *entry += plu.delta.abs() as u64
                }
            }
        }
    }
    pub fn get_price_levels(&self) -> BTreeMap<i8, u64> {
        self.prices.clone()
    }
}



