use crate::{
    ClientMessage, ClientMessageError, Command, RawMessage, Reply, ReplyError, Source, ToIrcLine,
};
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

impl ToIrcLine for Message {
    fn to_irc_line(&self) -> String {
        if let Some(ref src) = self.source {
            format!(":{src} {}", self.payload.to_irc_line())
        } else {
            self.payload.to_irc_line()
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Payload {
    ClientMessage(ClientMessage),
    Reply(Reply),
}

impl ToIrcLine for Payload {
    fn to_irc_line(&self) -> String {
        match self {
            Payload::ClientMessage(msg) => msg.to_irc_line(),
            Payload::Reply(r) => r.to_irc_line(),
        }
    }
}

#[derive(Debug, Error)]
pub enum MessageError {
    #[error(transparent)]
    ClientMessage(#[from] ClientMessageError),
    #[error(transparent)]
    Reply(#[from] ReplyError),
}
