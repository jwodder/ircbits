use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Quit;

impl ClientMessageParts for Quit {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<Quit> for Message {
    fn from(value: Quit) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Quit> for RawMessage {
    fn from(value: Quit) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Quit {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Quit, ClientMessageError> {
        todo!()
    }
}
