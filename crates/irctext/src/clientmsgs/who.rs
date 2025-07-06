use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Who;

impl ClientMessageParts for Who {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<Who> for Message {
    fn from(value: Who) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Who> for RawMessage {
    fn from(value: Who) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Who {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Who, ClientMessageError> {
        todo!()
    }
}
