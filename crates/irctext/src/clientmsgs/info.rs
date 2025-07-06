use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Info;

impl ClientMessageParts for Info {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<Info> for Message {
    fn from(value: Info) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Info> for RawMessage {
    fn from(value: Info) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Info {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Info, ClientMessageError> {
        todo!()
    }
}
