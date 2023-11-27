mod book;

use crate::book::get_book_data;
use actix::{Actor, StreamHandler};
use actix_web::web::Data;
use actix_web::{get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use actix_web_actors::ws::{Message, ProtocolError};
use fast_book::comm::urcp::{AddRequest, OBReqType, OBRequest, PriceViewResponse, read_response_vec, write_request};
use std::collections::BTreeMap;
use std::fmt::Write;
use std::ops::DerefMut;
use std::os::unix::net::UnixStream;
use std::sync::Mutex;

// TODO: remove this bullshit
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
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
        let mut stream = self.socket.lock().unwrap();
        let mut inner = stream.deref_mut();

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
                    "A" => {
                        debug_assert!(msg.len() == 4);
                        let qty = msg[1].parse::<u64>().unwrap();
                        let price = msg[2].parse::<i8>().unwrap();
                        let ob_id = msg[3].parse::<u16>().unwrap();
                        let req = AddRequest::new(qty, price, ob_id);
                        write_request(inner, &OBReqType::ADD, &OBRequest{ add: req }).unwrap();
                        let response_vec = read_response_vec(&mut inner).unwrap();

                        for response in response_vec.iter() {
                            println!("{:?}", response);
                        }
                    },
                    "V" => {
                        let oid = msg[1].parse::<u16>();
                        if oid.is_err() {
                            return;
                        }

                        let oid = oid.unwrap();
                        println!("oid: {}", oid);
                        let data = get_book_data(inner, oid);
                        match data {
                            Err(_) => {}
                            Ok(data) => {
                                println!("found some data\n");
                                let arr = data.prices;
                                // apriori allocation
                                let mut ret = String::with_capacity(arr.len() * (21) - 1);
                                ret.push_str("V:");

                                for (i, &num) in arr.iter().enumerate() {
                                    let price_level = (i as i16) - 100;

                                    if num > 0 {
                                        write!(ret, "{}:{};", price_level, num).expect("failed to write str");
                                    }
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
    let unix_stream: Data<Mutex<UnixStream>> = Data::new(Mutex::new(
        UnixStream::connect(STREAM_ADDR).expect("Couldn't connect to unix socket"),
    ));

    HttpServer::new(move || {
        App::new()
            .app_data(unix_stream.clone())
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
            .route("/ws/", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
