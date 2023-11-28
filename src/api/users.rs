use fast_book::comm::client::Client;

use actix_web::{get,post,Responder,HttpResponse};
use actix_web::web::Data;

use firebase_auth::FirebaseUser;

#[post("/user")]
pub async fn create_user(user: FirebaseUser, client: Data<Client>) -> impl Responder {
    HttpResponse::Ok().body("howdy")
}

#[get("/user")]
pub async fn get_user(user: FirebaseUser, client: Data<Client>) -> impl Responder {
    HttpResponse::Ok().body("howdy")
}
