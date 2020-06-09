use std::time::{Duration, Instant};

use actix::*;
use actix_web_actors::ws;
use log::debug;

use crate::types;
use crate::messages;
use crate::server::ChessServer;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct ChessSession {
      /// unique session id
      pub id: types::SessionId,
      /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
      /// otherwise we drop connection.
      pub hb: Instant,
      /// Chess server
      pub addr: Addr<ChessServer>,
}

impl Actor for ChessSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ChessSession in ChessServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in ChessServer. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        let addr = ctx.address();
        self.addr
            .send(messages::NewConnection {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chess server
        self.addr.do_send(messages::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from chess server, we simply send it to peer websocket
impl Handler<messages::OutMessage> for ChessSession {
    type Result = ();

    fn handle(&mut self, msg: messages::OutMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChessSession {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        debug!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {                
                // send message to ChessServer
                self.addr.do_send(messages::InMessage {
                    id: self.id,
                    content: text.trim().to_owned(),
                })
            }
            ws::Message::Binary(_) => debug!("Unexpected binary message received"),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

impl ChessSession {
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                debug!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                act.addr.do_send(messages::Disconnect { id: act.id });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}