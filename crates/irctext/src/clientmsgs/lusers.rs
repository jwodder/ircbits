use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lusers;

impl ClientMessageParts for Lusers {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<Lusers> for Message {
    fn from(value: Lusers) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Lusers> for RawMessage {
    fn from(value: Lusers) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Lusers {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Lusers, ClientMessageError> {
        todo!()
    }
}
