use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct List;

impl ClientMessageParts for List {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<List> for Message {
    fn from(value: List) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<List> for RawMessage {
    fn from(value: List) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for List {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<List, ClientMessageError> {
        todo!()
    }
}
