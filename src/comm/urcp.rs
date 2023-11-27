use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::io::Result;
use derive_more::Constructor;
use std::mem;

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    let ret = ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        mem::size_of::<T>(),
    );
    debug_assert_eq!(ret.len(), mem::size_of::<T>());
    ret
}

unsafe fn u8_slice_to_struct<T: Copy>(s: &[u8]) -> T {
    assert_eq!(s.len(), mem::size_of::<T>());

    // Create an unaligned reference to the slice and read it as a T
    let ptr = s.as_ptr() as *const T;
    ptr.read_unaligned()
}

// -------

#[derive(Clone,Copy)]
#[repr(u8)]
pub enum OBReqType {
    ADD = b'A',
    CANCEL = b'C',
    REDUCE = b'R',
    FLUSH = b'F',
    START = b'S',
    UNREACHABLE = b'-', // if i don't have it infinite loop bitches at me
}

impl OBReqType {
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
    pub fn from_u8(v: u8) -> Self {
        unsafe {
            mem::transmute(v)
        }
    }
}

pub struct OBRequestWrapper {
    pub req: OBRequest,
    pub typ: OBReqType
}

impl std::fmt::Debug for OBRequestWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.typ {
            OBReqType::ADD => unsafe { self.req.add.fmt(f) },
            OBReqType::CANCEL => unsafe { self.req.cancel.fmt(f) },
            OBReqType::REDUCE => unsafe { self.req.reduce.fmt(f) },
            OBReqType::FLUSH => unsafe { self.req.flush.fmt(f) },
            OBReqType::START => unsafe { self.req.start.fmt(f) },
            _ => f.write_str("unreachable")
        }
    }
}

#[derive(Copy,Clone)]
#[repr(C)]
pub union OBRequest {
    pub add: AddRequest,
    pub cancel: CancelRequest,
    pub reduce: ReduceRequest,
    pub flush: FlushRequest,
    pub start: StartRequest,
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct AddRequest {
    pub qty: u64,
    pub price: i8,
    pub ob_id: u16
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct CancelRequest {
    pub oid: usize,
    pub ob_id: u16
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct ReduceRequest {
    pub oid: usize,
    pub qty: u64,
    pub ob_id: u16
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct FlushRequest {
    pub ob_id: u16
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct StartRequest {
    pub ob_id: u16
}

pub fn write_request(stream: &mut UnixStream, typ: &OBReqType, req: &OBRequest) -> Result<()> {
    stream.write(&[typ.to_u8()])?;
    stream.write_all(unsafe { any_as_u8_slice::<OBRequest>(req) })?;
    Ok(())
}

pub fn read_request(stream: &mut UnixStream) -> Result<OBRequestWrapper> {
    let mut char_buf = [0u8; 1];
    stream.read_exact(&mut char_buf)?;
    let mut union_buf = [0u8; mem::size_of::<OBRequest>()];
    stream.read_exact(&mut union_buf)?;

    let union = unsafe { u8_slice_to_struct::<OBRequest>(&union_buf) };

    Ok(OBRequestWrapper{
        typ: OBReqType::from_u8(char_buf[0]),
        req: union
    })
}


// --------------------------
#[derive(Clone,Copy)]
#[repr(u8)]
pub enum OBRespType {
    ADD = b'A',
    EXECUTE = b'X',
    PRICE = b'$',
    DELIM = b'#',
}

impl OBRespType {
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
    pub fn from_u8(v: u8) -> Self {
        unsafe {
            std::mem::transmute(v)
        }
    }
}

#[repr(C)]
pub struct OBResponseWrapper {
    pub resp: OBResponse,
    pub typ: OBRespType
}

impl std::fmt::Debug for OBResponseWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.typ {
            OBRespType::ADD => unsafe { self.resp.add.fmt(f) },
            OBRespType::EXECUTE => unsafe { self.resp.execute.fmt(f) },
            OBRespType::PRICE => unsafe { self.resp.price.fmt(f) },
            OBRespType::DELIM => f.write_str("end of transmission"),
        }
    }
}

#[derive(Clone,Copy)]
#[repr(C)]
pub union OBResponse {
    pub add: AddResponse,
    pub execute: ExecuteResponse,
    pub price: PriceLevelResponse,
    pub end: DelimResponse
}


#[derive(Debug,Constructor,Clone,Copy)]
pub struct AddResponse {
    pub oid: usize
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct ExecuteResponse {
    pub executed_oid: usize,
    pub qty: u64
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct PriceLevelResponse {
    pub price: i8,
    pub delta: i64
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct DelimResponse {
}

impl PriceLevelResponse {
    pub fn from_pair(pair: (i8, i64)) -> Self {
        PriceLevelResponse {
            price: pair.0,
            delta: pair.1
        }
    }
}

pub fn write_response_vec(stream: &mut UnixStream, resps: Vec<OBResponseWrapper>) -> Result<()> {
    for resp in resps.iter() {
        write_response(stream, &resp.typ, &resp.resp)?;
    }

    write_response(stream, &OBRespType::DELIM, &OBResponse{ end: DelimResponse{} })?;
    Ok(())
}

pub fn write_response(stream: &mut UnixStream, typ: &OBRespType, data: &OBResponse) -> Result<()> {
    let u8_slice = unsafe { any_as_u8_slice::<OBResponse>(data) };
    stream.write(&[typ.to_u8()])?;
    stream.write_all(&u8_slice)?;
    Ok(())
}

pub fn read_response(stream: &mut UnixStream) -> Result<OBResponseWrapper> {
    let mut char_buf = [0u8; 1];
    stream.read_exact(&mut char_buf)?;
    let mut union_buf = [0u8; mem::size_of::<OBResponse>()];
    stream.read_exact(&mut union_buf)?;

    let union = unsafe { u8_slice_to_struct::<OBResponse>(&union_buf) };

    Ok(OBResponseWrapper{
        typ: OBRespType::from_u8(char_buf[0]),
        resp: union
    })
}

pub fn read_response_vec(stream: &mut UnixStream) -> Result<Vec<OBResponseWrapper>> {
    let mut ret: Vec<OBResponseWrapper> = Vec::new();

    loop {
        let response = read_response(stream)?;
        match response.typ {
            OBRespType::DELIM => break,
            _ => ret.push(response),
        }
    }
    Ok(ret)
}
