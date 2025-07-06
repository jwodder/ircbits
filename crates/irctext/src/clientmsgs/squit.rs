use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Squit;

impl ClientMessageParts for Squit {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Squit {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Squit> for Message {
    fn from(value: Squit) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Squit> for RawMessage {
    fn from(value: Squit) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Squit {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Squit, ClientMessageError> {
        todo!()
    }
}
