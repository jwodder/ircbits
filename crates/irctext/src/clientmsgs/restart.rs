use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Restart;

impl ClientMessageParts for Restart {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<Restart> for Message {
    fn from(value: Restart) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Restart> for RawMessage {
    fn from(value: Restart) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Restart {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Restart, ClientMessageError> {
        todo!()
    }
}
