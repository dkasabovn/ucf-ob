extern crate fast_book;

use fast_book::comm::urcp::*;

use std::io::Result;
use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::mem;

const STREAM_ADDR: &'static str = "/tmp/fish.socket";

fn main() -> Result<()> {
    let mut listener = UnixStream::connect(STREAM_ADDR)?;

    loop {
        let mut input = String::new();

        std::io::stdin().read_line(&mut input).unwrap();

        let mut inputs: Vec<&str> = input.split_whitespace().collect();
        
        let cmd = inputs.remove(0);

        let mast = cmd.chars().next().unwrap();


        match mast {
            'A' => {
                debug_assert!(inputs.len() == 3);
                let qty = inputs[0].parse::<u64>().unwrap();
                let price = inputs[1].parse::<i8>().unwrap();
                let ob_id = inputs[2].parse::<u16>().unwrap();
                let req = AddRequest::new(qty, price, ob_id);
                write_request(&mut listener, &OBReqType::ADD, &OBRequest{ add: req })?;
                let response_vec = read_response_vec(&mut listener)?;

                for response in response_vec.iter() {
                    println!("{:?}", response);
                }
            },
            'C' => {
                debug_assert!(inputs.len() == 2);
                let oid = inputs[0].parse::<usize>().unwrap();
                let ob_id = inputs[1].parse::<u16>().unwrap();
                let req = CancelRequest::new(oid, ob_id);
                write_request(&mut listener, &OBReqType::CANCEL, &OBRequest{ cancel: req })?;
                let response = read_response(&mut listener)?;
                println!("{:?}", response);
            },
            'R' => {
                debug_assert!(inputs.len() == 3);
                let oid = inputs[0].parse::<usize>().unwrap();
                let qty = inputs[1].parse::<u64>().unwrap();
                let ob_id = inputs[2].parse::<u16>().unwrap();
                let req = ReduceRequest::new(oid, qty, ob_id);
                write_request(&mut listener, &OBReqType::REDUCE, &OBRequest{ reduce: req })?;
                let response = read_response(&mut listener)?;
                println!("{:?}", response);
            },
            'F' => {
                debug_assert!(inputs.len() == 1);
                let ob_id = inputs[0].parse::<u16>().unwrap();
                let req = FlushRequest::new(ob_id);
                write_request(&mut listener, &OBReqType::FLUSH, &OBRequest{ flush: req })?;
                let response = read_response(&mut listener)?;
                println!("{:?}", response);
            },
            'S' => {
                debug_assert!(inputs.len() == 1);
                let ob_id = inputs[0].parse::<u16>().unwrap();
                let req = StartRequest::new(ob_id);
                write_request(&mut listener, &OBReqType::START, &OBRequest{ start: req })?;
                let response = read_response(&mut listener)?;
                println!("{:?}", response);
            },
            _ => {
                listener.shutdown(std::net::Shutdown::Both)?;
                break;
            },
        }
    }
    Ok(())
}
