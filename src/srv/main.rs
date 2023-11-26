extern crate fast_book;

use fast_book::comm::urcp::*;
use fast_book::comm::manager::*;

use std::io::prelude::*;
use std::io::Result;
use std::mem;
use std::os::unix::net::*;

const ORDER_SIZE: usize = 1000000;
const LEVEL_SIZE: usize = 200;
const BOOKS: u16 = 1;

const STREAM_ADDR: &'static str = "/tmp/fish.socket";

unsafe fn u8_slice_to_struct<T: Copy>(s: &[u8]) -> T {
    assert_eq!(s.len(), mem::size_of::<T>());

    // Create an unaligned reference to the slice and read it as a T
    let ptr = s.as_ptr() as *const T;
    ptr.read_unaligned()
}

fn main() -> Result<()> {
    let mut manager = Manager::new(ORDER_SIZE, LEVEL_SIZE, BOOKS);

    let _ = std::fs::remove_file(STREAM_ADDR);

    let server = UnixListener::bind(STREAM_ADDR)?;

    let mut listener = server.accept().unwrap().0;

    listener.set_read_timeout(None)?;

    let mut buffer = [0u8; 1];
    loop {
        listener.read_exact(&mut buffer)?;

        match char::from(buffer[0]) {
            'A' => {
                let mut add_buffer = [0u8; mem::size_of::<AddRequest>()];
                listener.read_exact(&mut add_buffer)?;
                let add_op = unsafe { u8_slice_to_struct::<AddRequest>(&add_buffer) };
                println!("adding {:?}", add_op);

                manager[add_op.ob_id as usize].add(add_op.qty, add_op.ob_id, add_op.price);
            }
            'C' => {
                let mut cancel_buffer = [0u8; mem::size_of::<CancelRequest>()];
                listener.read_exact(&mut cancel_buffer)?;
                let cancel_op = unsafe { u8_slice_to_struct::<CancelRequest>(&cancel_buffer) };
                println!("cancelling {:?}", cancel_op);

                manager[cancel_op.ob_id as usize].delete(cancel_op.oid);
            }
            'R' => {
                let mut reduce_buffer = [0u8; mem::size_of::<ReduceRequest>()];
                listener.read_exact(&mut reduce_buffer)?;
                let reduce_op = unsafe { u8_slice_to_struct::<ReduceRequest>(&reduce_buffer) };
                println!("reducing {:?}", reduce_op);
            }
            'F' => {
                let mut flush_buffer = [0u8; mem::size_of::<FlushRequest>()];
                listener.read_exact(&mut flush_buffer)?;
                let flush_op = unsafe { u8_slice_to_struct::<FlushRequest>(&flush_buffer) };
                println!("stop trading and flush orderbook {:?}", flush_op);
            }
            'S' => {
                let mut start_buffer = [0u8; mem::size_of::<StartRequest>()];
                listener.read_exact(&mut start_buffer)?;
                let start_op = unsafe { u8_slice_to_struct::<StartRequest>(&start_buffer) };
                println!("start orderbook {:?}", start_op);
            },
            'P' => {
                manager[0].print();
            },
            '\0' => {
                break;
            }
            _ => todo!(),
        }
    }

    Ok(())
}
