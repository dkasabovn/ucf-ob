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
