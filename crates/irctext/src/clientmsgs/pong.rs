use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pong;

impl ClientMessageParts for Pong {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Pong {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Pong> for Message {
    fn from(value: Pong) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Pong> for RawMessage {
    fn from(value: Pong) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Pong {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Pong, ClientMessageError> {
        todo!()
    }
}
