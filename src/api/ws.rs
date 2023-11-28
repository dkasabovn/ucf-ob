use actix::{Actor, StreamHandler, AsyncContext, spawn};
use actix_web::{web, HttpResponse, HttpRequest, Error};
use actix_web_actors::ws;
use tokio::time::{Duration};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use serde_json::to_string;
use futures::StreamExt;

use fast_book::comm::domain::*;
use fast_book::comm::client::Client;


struct FTXWS {
    broadcaster: broadcast::Sender<String>,
    client: Client,
}

struct BroadcastMessage(String);

impl actix::Message for BroadcastMessage {
    type Result = ();
}

impl actix::Handler<BroadcastMessage> for FTXWS {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0)
    }
}

impl Actor for FTXWS {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_millis(250), move |ftxws, ctx| {
            let ob_levels = ftxws.client.get_ob_levels();
            let pkg = ApiViewResponse {
                typ: String::from("view"),
                data: ob_levels
            };
            if let Ok(json) = to_string(&pkg) {
                ctx.text(json);
            }
        });

        let mut receiver = BroadcastStream::new(self.broadcaster.subscribe());
        let addr = ctx.address();
        spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                addr.do_send(BroadcastMessage(msg));
            }
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for FTXWS {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            _ => (),
        }
    }
}

pub async fn websocket_route(req: HttpRequest, stream: web::Payload, broadcaster: web::Data<broadcast::Sender<String>>, client: web::Data<Client>) -> Result<HttpResponse, Error> {
    let ws = FTXWS {
        broadcaster: broadcaster.get_ref().clone(),
        client: client.get_ref().clone(),
    };
    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
