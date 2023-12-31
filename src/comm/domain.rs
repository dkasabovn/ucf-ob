use serde::Serialize;

#[derive(Serialize)]
pub struct User {
    pub id: i32,
    pub sub: String,
    pub balance: i32
}

#[derive(Serialize)]
pub struct Contract {
    pub book_id: i32,
    pub yes_holder: i32,
    pub no_holder: i32,
    pub qty: i32,
}

#[derive(Serialize,Debug)]
pub struct UserOrder {
    pub id: i32,
    pub book_id: i32,
    pub price: i32,
    pub qty: i32,
    pub user_fk: i32,
}

#[derive(Serialize)]
pub struct GenericResponse {
    pub msg: String
}

#[derive(Serialize)]
pub struct ApiViewResponse {
    pub typ: String,
    pub data: Vec<std::collections::BTreeMap<i8, u64>>
}

#[derive(Serialize)]
pub struct ApiViewInner {
    pub yes: Vec<(i8, u64)>,
    pub no: Vec<(i8, u64)>
}

#[derive(Serialize)]
pub struct ApiExecuteResponse {
    pub typ: String,
    pub data: ApiExecuteInner
}

#[derive(Serialize)]
pub struct ApiExecuteInner {
    pub oid: usize,
    pub qty: u64
}

#[derive(Serialize)]
pub struct ApiResultResponse {
    pub top: bool,
    pub right: bool,
}
