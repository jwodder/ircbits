use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Authenticate;

impl ClientMessageParts for Authenticate {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<Authenticate> for Message {
    fn from(value: Authenticate) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Authenticate> for RawMessage {
    fn from(value: Authenticate) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Authenticate {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Authenticate, ClientMessageError> {
        todo!()
    }
}
