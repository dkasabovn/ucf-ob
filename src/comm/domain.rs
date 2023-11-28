pub struct User {
    pub id: i32,
    pub sub: String,
    pub balance: i32
}

pub struct Contract {
    pub yes_holder: i32,
    pub no_holder: i32,
    pub qty: i32,
}

pub struct UserOrder {
    pub id: i32,
    pub book_id: i32,
    pub price: i32,
    pub qty: i32,
    pub user_fk: i32,
}
