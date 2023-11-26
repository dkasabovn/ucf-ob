pub struct Order {
    side: Intent,
    price: i8,
    symbol: u16,
    quantity: u64
}

pub enum Intent {
    BID,
    ASK
}

pub struct Level {
    side: Intent,
    price: i8,
    volume: i8,
    orders: usize
}
