use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Userhost;

impl ClientMessageParts for Userhost {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<Userhost> for Message {
    fn from(value: Userhost) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Userhost> for RawMessage {
    fn from(value: Userhost) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Userhost {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Userhost, ClientMessageError> {
        todo!()
    }
}
