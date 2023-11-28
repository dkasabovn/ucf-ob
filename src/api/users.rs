use fast_book::comm::client::Client;
use fast_book::comm::domain::*;

use actix_web::{get,post,Responder,HttpResponse};
use actix_web::web::Data;

use firebase_auth::FirebaseUser;

#[post("/user")]
pub async fn create_user(user: FirebaseUser, client: Data<Client>) -> impl Responder {
    match client.create_user(user.sub) {
        Some(_) => HttpResponse::Ok().json(GenericResponse{msg: "ok".to_string()}),
        None => HttpResponse::BadRequest().json(GenericResponse{msg: "err".to_string()})
    }
}

#[get("/user")]
pub async fn get_user(user: FirebaseUser, client: Data<Client>) -> impl Responder {
    match client.get_user(user.sub) {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::NotFound().json(GenericResponse{msg: "err".to_string()})
    }
}
