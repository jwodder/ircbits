use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rehash;

impl ClientMessageParts for Rehash {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<Rehash> for Message {
    fn from(value: Rehash) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Rehash> for RawMessage {
    fn from(value: Rehash) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Rehash {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Rehash, ClientMessageError> {
        todo!()
    }
}
