use fast_book::comm::client::Client;

use actix_web::{get,post,delete,put,Responder,HttpResponse};
use actix_web::web::Data;

use firebase_auth::FirebaseUser;

#[post("/order")]
pub async fn create_order(user: FirebaseUser, client: Data<Client>) -> impl Responder {
    HttpResponse::Ok().body("howdy")
}

#[get("/orders")]
pub async fn get_orders(user: FirebaseUser, client: Data<Client>) -> impl Responder {
    HttpResponse::Ok().body("howdy")
}

#[delete("/order/{oid}")]
pub async fn delete_order(user: FirebaseUser, client: Data<Client>) -> impl Responder {
    HttpResponse::Ok().body("howdy")
}

#[put("/order/{oid}")]
pub async fn reduce_order(user: FirebaseUser, client: Data<Client>) -> impl Responder {
    HttpResponse::Ok().body("howdy")
}
