use std::time::Instant;

use actix::Addr;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::server::ChessServer;
use crate::session::ChessSession;

pub async fn session_handler(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<ChessServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        ChessSession {
            id: 0,
            hb: Instant::now(),
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}