use std::collections::HashMap;

use actix::{Actor, Context, Handler, Recipient};
use log::debug;

use crate::types;
use crate::messages;
use super::generator::SessionIdGenerator;

pub struct ChessServer {
    sessions: HashMap<types::SessionId, Recipient<messages::OutMessage>>,
    session_id_gen: SessionIdGenerator,
}

impl Default for ChessServer {
    fn default() -> ChessServer {
        ChessServer {
            sessions: HashMap::new(),
            session_id_gen: Default::default(),
        }
    }
}

impl ChessServer {
    // broadcast message to everyone
    fn send_message_broadcast(&self, message: &str, sender_id: types::SessionId) {
        for (id, session) in &self.sessions {
            if *id != sender_id {
                session.do_send(
                    messages::OutMessage(message.to_owned())
                ).ok();
            }
        }
    }

    // send message to one session
    fn send_message(&self, message: &str, recipient_id: types::SessionId) {
        if let Some(session) = self.sessions.get(&recipient_id) {
            session.do_send(
                messages::OutMessage(message.to_owned())
            ).ok();
        }
    }
}

impl Actor for ChessServer {
    type Context = Context<Self>;
}

impl Handler<messages::NewConnection> for ChessServer {
    type Result = types::SessionId;

    fn handle(&mut self, msg: messages::NewConnection, _: &mut Context<Self>) -> Self::Result {
        debug!("New connection");

        // register the session
        let id = self.session_id_gen.generate();
        self.sessions.insert(id, msg.addr);

        // Welcome the user
        self.send_message("Welcome", id);
        // notify other users
        self.send_message_broadcast("Someone joined", id);

        // send id back
        id
    }
}

impl Handler<messages::Disconnect> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: messages::Disconnect, _: &mut Context<Self>) {
        debug!("Disconnect");

        self.sessions.remove(&msg.id);
        // send message to other users
        self.send_message_broadcast("Someone disconnected", 0);
    }
}

/// Handler for received message.
impl Handler<messages::InMessage> for ChessServer {
    type Result = ();

    fn handle(&mut self, msg: messages::InMessage, _: &mut Context<Self>) {
        self.send_message_broadcast(msg.content.as_str(), msg.id);
    }
}