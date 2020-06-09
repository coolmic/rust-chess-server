use actix::prelude::{Message, Recipient};

use crate::types;

#[derive(Message)]
#[rtype(result = "()")]
pub struct OutMessage(pub String);

#[derive(Message)]
#[rtype(usize)]
pub struct NewConnection {
    pub addr: Recipient<OutMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: types::SessionId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct InMessage {
    pub id: types::SessionId,
    pub content: String,
}