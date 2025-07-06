use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoWas;

impl ClientMessageParts for WhoWas {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<WhoWas> for Message {
    fn from(value: WhoWas) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<WhoWas> for RawMessage {
    fn from(value: WhoWas) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for WhoWas {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<WhoWas, ClientMessageError> {
        todo!()
    }
}
