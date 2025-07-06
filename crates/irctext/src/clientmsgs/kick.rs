use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Kick;

impl ClientMessageParts for Kick {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<Kick> for Message {
    fn from(value: Kick) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Kick> for RawMessage {
    fn from(value: Kick) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Kick {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Kick, ClientMessageError> {
        todo!()
    }
}
