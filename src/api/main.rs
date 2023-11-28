mod book;
mod users;
mod order;

use crate::book::get_book_data;
use actix::{Actor, StreamHandler};
use actix_web::web::Data;
use actix_web::{get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use actix_web_actors::ws::{Message, ProtocolError};
use fast_book::comm::urcp::PriceViewResponse;
use std::fmt::Write;
use std::os::unix::net::UnixStream;
use std::sync::Mutex;

// TODO: remove this bullshit
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/result")]
async fn result() -> impl Responder {
    HttpResponse::Ok().body("okayga")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

struct MyWs {
    socket: Data<Mutex<UnixStream>>,
}
impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<Message, ProtocolError>> for MyWs {
    fn handle(&mut self, item: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(Message::Ping(msg)) => ctx.pong(&msg),
            Ok(Message::Text(msg)) => {
                // handle with unix requests
                // Message should follow format: <type> <oid>
                let msg = msg.split(' ');
                let msg: Vec<&str> = msg.collect();
                if msg.is_empty() {
                    return;
                }

                match msg[0] {
                    "V" => {
                        let oid = msg[1].parse::<u16>();
                        if oid.is_err() {
                            return;
                        }

                        let oid = oid.unwrap();
                        let data = get_book_data(self.socket.clone(), oid);
                        match data {
                            None => {}
                            Some(data) => {
                                let arr = data.prices;
                                // apriori allocation
                                let mut ret = String::with_capacity(arr.len() * (21) - 1);

                                for (i, &num) in arr.iter().enumerate() {
                                    if i > 0 {
                                        ret.push(',');
                                    }
                                    write!(ret, "{}", num).expect("failed to write str");
                                }

                                ctx.text(ret);
                                return;
                            }
                        }
                    }
                    &_ => {}
                }
            }
            Ok(Message::Binary(msg)) => ctx.binary(msg),
            _ => (),
        }
    }
}

async fn index(
    socket: Data<Mutex<UnixStream>>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(MyWs { socket }, &req, stream);
    println!("{:?}", resp);
    resp
}

const STREAM_ADDR: &'static str = "/tmp/fish.socket";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //let unix_stream: Data<Mutex<UnixStream>> = Data::new(Mutex::new(
    //    UnixStream::connect(STREAM_ADDR).expect("Couldn't connect to unix socket"),
    //));

    HttpServer::new(move || {
        App::new()
            .service(hello)
            .service(echo)
            .service(result)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
