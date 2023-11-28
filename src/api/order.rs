use fast_book::comm::client::Client;

use actix_web::{get,post,delete,put,Responder,HttpResponse,web};
use actix_web::web::Data;

use serde::{Serialize,Deserialize};

use firebase_auth::FirebaseUser;

#[derive(Serialize,Deserialize)]
pub struct CreateOrder {
    qty: u64,
    price: i8,
    market: u16,
    yes: bool
}

#[post("/order")]
pub async fn create_order(user: FirebaseUser, client: Data<Client>, payload: web::Json<CreateOrder>) -> impl Responder {
    let user = match client.get_user(user.sub) {
        Some(user) => user,
        _ => return HttpResponse::Unauthorized().body("howdy"),
    };

    // we only have 2 markets
    assert!(payload.market < 2);
    assert!(payload.price > 0 && payload.price < 100);
    assert!(payload.qty > 0);

    let ip = if payload.yes {
        -payload.price
    } else {
        100 - payload.price
    };

    match client.add_order(&user, ip, payload.qty, payload.market) {
        Some(add_response) => HttpResponse::Ok().json(add_response),
        None => return HttpResponse::BadRequest().body("bad request not enough schmoney"),
    }
}

#[get("/orders")]
pub async fn get_orders(user: FirebaseUser, client: Data<Client>) -> impl Responder {
    let user = match client.get_user(user.sub) {
        Some(user) => user,
        _ => return HttpResponse::Unauthorized().body("howdy"),
    };

    match client.get_orders(&user) {
        Some(orders) => HttpResponse::Ok().json(orders),
        _ => HttpResponse::NotFound().body("not found"),
    }
}

#[delete("/order/{oid}/{market}")]
pub async fn delete_order(user: FirebaseUser, client: Data<Client>, payload: web::Path<(usize,u16)>) -> impl Responder {
    let user = match client.get_user(user.sub) {
        Some(user) => user,
        _ => return HttpResponse::Unauthorized().body("howdy"),
    };

    let (oid, market) = payload.into_inner();

    assert!(market < 2);

    match client.cancel_order(&user, oid, market) {
        Some(_) => HttpResponse::Ok().body("success"),
        None => HttpResponse::InternalServerError().body("failure"),
    }
}

#[derive(Serialize,Deserialize)]
pub struct ModifyOrder {
    qty: u64,
    oid: usize,
    market: u16,
}

#[put("/order")]
pub async fn reduce_order(user: FirebaseUser, client: Data<Client>, payload: web::Json<ModifyOrder>) -> impl Responder {
    let user = match client.get_user(user.sub) {
        Some(user) => user,
        _ => return HttpResponse::Unauthorized().body("howdy"),
    };

    match client.reduce_order(&user, payload.oid, payload.qty, payload.market) {
        Some(_) => HttpResponse::Ok().body("success"),
        None => HttpResponse::InternalServerError().body("failure"),
    }
}

#[put("/result/{result_quad}")]
pub async fn set_result(client: Data<Client>, payload: web::Path<usize>) -> impl Responder {
    let bit_repr = payload.into_inner();

    let top: bool = bit_repr & 2 > 0;
    let right: bool = bit_repr & 1 > 0;

    HttpResponse::Ok().body("success")
}
