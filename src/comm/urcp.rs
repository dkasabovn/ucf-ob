use derive_more::Constructor;


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


// --------------------------
#[derive(Clone,Copy)]
pub enum MatchingType {
    ADD = 'A' as isize,
    EXECUTE = 'X' as isize,
    PRICE = '$' as isize,
    CANCEL = 'C' as isize,
    REDUCE = 'R' as isize,
}

impl MatchingType {
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

pub struct MatchingWrapper {
    pub resp: MatchingResponse,
    pub typ: MatchingType
}

#[repr(C)]
#[derive(Clone,Copy)]
pub union MatchingResponse {
    pub add: AddResponse,
    pub execute: ExecuteResponse,
    pub price: PriceLevelResponse,
    pub cancel: CancelResponse,
    pub reduce: ReduceResponse
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
pub struct CancelResponse {
    pub oid: usize,
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct ReduceResponse {
    pub oid: usize,
    pub qty: usize
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct PriceLevelResponse {
    pub price: i8,
    pub volume: u64
}

