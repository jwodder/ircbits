use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Whowas;

impl ClientMessageParts for Whowas {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<Whowas> for Message {
    fn from(value: Whowas) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Whowas> for RawMessage {
    fn from(value: Whowas) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Whowas {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Whowas, ClientMessageError> {
        todo!()
    }
}
