use std::env;

use dotenv::dotenv;
use log::warn;
use actix::prelude::Actor;
use actix_web::{web, App, HttpServer};

mod types;
mod messages;
mod server;
mod session;

use crate::server::ChessServer;
use crate::session::session_handler;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let addr = match env::var("APP_BIND_ADDR") {
        Ok(addr) => addr,
        Err(_) => {
            warn!("env APP_BIND_ADDR not defined, use default port addr 127.0.0.1:8080");
            "127.0.0.1:8080".to_owned()
        },
    };

    let server = ChessServer::default().start();

    // Create Http server with websocket support
    HttpServer::new(move || {
        App::new()
            .data(server.clone())
            .service(web::resource("/").to(session_handler))
    })
    .bind(addr)?
    .run()
    .await
}
