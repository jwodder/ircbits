use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Connect;

impl ClientMessageParts for Connect {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<Connect> for Message {
    fn from(value: Connect) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Connect> for RawMessage {
    fn from(value: Connect) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Connect {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Connect, ClientMessageError> {
        todo!()
    }
}
