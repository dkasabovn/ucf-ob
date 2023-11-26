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
    pub volume: u64
}
