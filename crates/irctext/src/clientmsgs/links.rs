use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Links;

impl ClientMessageParts for Links {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Links {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Links> for Message {
    fn from(value: Links) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Links> for RawMessage {
    fn from(value: Links) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Links {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Links, ClientMessageError> {
        todo!()
    }
}
