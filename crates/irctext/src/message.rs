use crate::{ClientMessage, ClientMessageError, Command, RawMessage, Reply, ReplyError, Source};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub source: Option<Source>,
    pub payload: Payload,
}

impl TryFrom<RawMessage> for Message {
    type Error = MessageError;

    fn try_from(msg: RawMessage) -> Result<Message, MessageError> {
        let source = msg.source;
        let payload = match msg.command {
            Command::Verb(v) => {
                Payload::ClientMessage(ClientMessage::from_parts(v, msg.parameters)?)
            }
            Command::Reply(code) => Payload::Reply(Reply::from_parts(code, msg.parameters)?),
        };
        Ok(Message { source, payload })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Payload {
    ClientMessage(ClientMessage),
    Reply(Reply),
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum MessageError {
    #[error(transparent)]
    ClientMessage(#[from] ClientMessageError),
    #[error(transparent)]
    Reply(#[from] ReplyError),
}
