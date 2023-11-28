mod users;
mod order;
mod ws;

use actix_web::{get, post, App, HttpResponse, HttpServer, Responder, web};
use actix_cors::Cors;
use tokio::sync::broadcast;

use fast_book::comm::client::Client;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("FTX.markets API. texasucf.org")
}

#[get("/result")]
async fn result() -> impl Responder {
    HttpResponse::Ok().body("Insider trading already? Smh")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

const STREAM_ADDR: &'static str = "/tmp/fish.socket";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let firebase_auth = firebase_auth::FirebaseAuth::new("bets-fc705").await;
    let (tx, _rx) = broadcast::channel::<String>(100);
    let client = Client::new(STREAM_ADDR, tx.clone())?;


    HttpServer::new(move || {
        
        let cors = Cors::default()
            .allow_any_method()
            .allow_any_header()
            .allow_any_origin();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(firebase_auth.clone()))
            .app_data(web::Data::new(tx.clone()))
            .app_data(web::Data::new(client.clone()))
            .service(hello)
            .service(echo)
            .service(result)
            .service(users::get_user)
            .service(users::create_user)
            .service(order::get_orders)
            .service(order::get_orders_satisfied)
            .service(order::create_order)
            .service(order::delete_order)
            .service(order::reduce_order)
            .route("/ws/", web::get().to(ws::websocket_route))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
