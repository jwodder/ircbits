use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ping;

impl ClientMessageParts for Ping {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<Ping> for Message {
    fn from(value: Ping) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Ping> for RawMessage {
    fn from(value: Ping) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Ping {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Ping, ClientMessageError> {
        todo!()
    }
}
