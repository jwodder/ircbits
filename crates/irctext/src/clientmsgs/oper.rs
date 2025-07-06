use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Oper;

impl ClientMessageParts for Oper {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<Oper> for Message {
    fn from(value: Oper) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Oper> for RawMessage {
    fn from(value: Oper) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Oper {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Oper, ClientMessageError> {
        todo!()
    }
}
