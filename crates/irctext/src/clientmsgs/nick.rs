use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Nick;

impl ClientMessageParts for Nick {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Nick {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Nick> for Message {
    fn from(value: Nick) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Nick> for RawMessage {
    fn from(value: Nick) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Nick {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Nick, ClientMessageError> {
        todo!()
    }
}
