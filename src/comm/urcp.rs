use derive_more::Constructor;

#[derive(Debug,Constructor,Clone,Copy)]
pub struct AddRequest {
    qty: u64,
    price: i8,
    ob_id: u16
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct CancelRequest {
    oid: usize,
    ob_id: u16
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct ReduceRequest {
    oid: usize,
    qty: u64,
    ob_id: u16
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct FlushRequest {
    ob_id: u16
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct StartRequest {
    ob_id: u16
}


// --------------------------

#[derive(Debug,Constructor,Clone,Copy)]
pub struct AddResponse {
    oid: usize
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct ExecuteResponse {
    executed_oid: usize,
    qty: u64
}

#[derive(Debug,Constructor,Clone,Copy)]
pub struct PriceLevelResponse {
    price: i8,
    volume: u64
}
