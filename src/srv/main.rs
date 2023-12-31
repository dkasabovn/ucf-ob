extern crate fast_book;

use fast_book::comm::urcp::*;
use fast_book::comm::manager::*;

use std::io::Result;
use std::os::unix::net::*;

const ORDER_SIZE: usize = 1000000;
const LEVEL_SIZE: usize = 200;
const BOOKS: u16 = 2;

const STREAM_ADDR: &'static str = "/tmp/fish.socket";


fn main() -> Result<()> {
    let mut manager = Manager::new(ORDER_SIZE, LEVEL_SIZE, BOOKS);

    let _ = std::fs::remove_file(STREAM_ADDR);

    let server = UnixListener::bind(STREAM_ADDR)?;

    let mut listener = server.accept().unwrap().0;

    listener.set_read_timeout(None)?;
    listener.set_write_timeout(None)?;
    listener.set_nonblocking(false)?;

    loop {
        let request = read_request(&mut listener)?;
        println!("{:?}", request);
        unsafe {
            match request {
                OBRequestWrapper { req: OBRequest { add: req }, typ: OBReqType::ADD } => {
                    let response_vec = manager[req.ob_id as usize].match_order(req.qty, req.price);
                    write_response_vec(&mut listener, response_vec)?;
                },
                OBRequestWrapper { req: OBRequest { cancel: req }, typ: OBReqType::CANCEL } => {
                    let price_level_response = manager[req.ob_id as usize].delete(req.oid);
                    write_response(&mut listener, &OBRespType::PRICE, &OBResponse { price: price_level_response })?;
                },
                OBRequestWrapper { req: OBRequest { reduce: req }, typ: OBReqType::REDUCE } => {
                    let price_level_response = manager[req.ob_id as usize].reduce(req.oid, req.qty);
                    write_response(&mut listener, &OBRespType::PRICE, &OBResponse { price: price_level_response })?;
                },
                OBRequestWrapper { req: OBRequest { flush: req }, typ: OBReqType::FLUSH } => {
                    manager[req.ob_id as usize].clear();
                    write_response(&mut listener, &OBRespType::DELIM, &OBResponse { end: DelimResponse{}})?;
                },
                _ => break
            };
        };
    }

    unreachable!("malformatted request");
}
