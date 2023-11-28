mod book;
mod users;
mod order;
mod ws;

use actix_web::{get, post, App, HttpResponse, HttpServer, Responder, web};
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
    let firebase_auth = firebase_auth::FirebaseAuth::new("project-id").await;
    let (tx, _) = broadcast::channel::<String>(100);
    let client = Client::new(STREAM_ADDR, tx.clone())?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(firebase_auth.clone()))
            .app_data(web::Data::new(tx.clone()))
            .app_data(web::Data::new(client.clone()))
            .service(hello)
            .service(echo)
            .service(result)
            .service(users::get_user)
            .service(users::create_user)
            .route("/ws/", web::get().to(ws::websocket_route))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
