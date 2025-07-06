use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Part;

impl ClientMessageParts for Part {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Part {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Part> for Message {
    fn from(value: Part) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Part> for RawMessage {
    fn from(value: Part) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Part {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Part, ClientMessageError> {
        todo!()
    }
}
