extern crate fast_book;

use fast_book::comm::urcp::*;

use std::io::Result;
use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::mem;

const STREAM_ADDR: &'static str = "/tmp/fish.socket";

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
}

unsafe fn u8_slice_as_any<T: Sized>(s: &[u8]) -> &T {
    debug_assert_eq!(s.len(), mem::size_of::<T>());
    &*(s.as_ptr() as *const T)
}

fn main() -> Result<()> {
    let mut listener = UnixStream::connect(STREAM_ADDR)?;

    loop {
        let mut input = String::new();

        std::io::stdin().read_line(&mut input).unwrap();

        let mut inputs: Vec<&str> = input.split_whitespace().collect();
        
        let cmd = inputs.remove(0);

        let mast = cmd.chars().next().unwrap();

        let mut char_buf = [0u8; 1];

        match mast {
            'A' => {
                char_buf[0] = 'A' as u8;

                debug_assert!(inputs.len() == 3);
                let qty = inputs[0].parse::<u64>().unwrap();
                let price = inputs[1].parse::<i8>().unwrap();
                let ob_id = inputs[2].parse::<u16>().unwrap();
                let req = AddRequest::new(qty, price, ob_id);
                let sloice = unsafe { any_as_u8_slice::<AddRequest>(&req) };


                listener.write(&char_buf)?;
                listener.write_all(&sloice)?;
                println!("wrote {:?} to socket", req);
            },
            'C' => {
                char_buf[0] = 'C' as u8;

                debug_assert!(inputs.len() == 2);
                let oid = inputs[0].parse::<usize>().unwrap();
                let ob_id = inputs[1].parse::<u16>().unwrap();
                let req = CancelRequest::new(oid, ob_id);
                let sloice = unsafe { any_as_u8_slice::<CancelRequest>(&req) };


                listener.write(&char_buf)?;
                listener.write_all(&sloice)?;
                println!("wrote {:?} to socket", req);

            },
            'R' => {
                char_buf[0] = 'R' as u8;

                debug_assert!(inputs.len() == 3);
                let oid = inputs[0].parse::<usize>().unwrap();
                let qty = inputs[1].parse::<u64>().unwrap();
                let ob_id = inputs[2].parse::<u16>().unwrap();
                let req = ReduceRequest::new(oid, qty, ob_id);
                let sloice = unsafe { any_as_u8_slice::<ReduceRequest>(&req) };


                listener.write(&char_buf)?;
                listener.write_all(&sloice)?;
                println!("wrote {:?} to socket", req);
            },
            'F' => {
                char_buf[0] = 'F' as u8;

                debug_assert!(inputs.len() == 1);
                let ob_id = inputs[0].parse::<u16>().unwrap();
                let req = FlushRequest::new(ob_id);
                let sloice = unsafe { any_as_u8_slice::<FlushRequest>(&req) };


                listener.write(&char_buf)?;
                listener.write_all(&sloice)?;
                println!("wrote {:?} to socket", req);
            },
            'S' => {
                char_buf[0] = 'S' as u8;

                debug_assert!(inputs.len() == 1);
                let ob_id = inputs[0].parse::<u16>().unwrap();
                let req = FlushRequest::new(ob_id);
                let sloice = unsafe { any_as_u8_slice::<FlushRequest>(&req) };


                listener.write(&char_buf)?;
                listener.write_all(&sloice)?;
                println!("wrote {:?} to socket", req);
            },
            'P' => {
                char_buf[0] = 'P' as u8;
                listener.write(&char_buf)?;
                println!("sent print request");
            },
            _ => {
                listener.shutdown(std::net::Shutdown::Both)?;
                break;
            },
        }
    }
    Ok(())
}
