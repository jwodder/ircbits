use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Names;

impl ClientMessageParts for Names {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Names {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Names> for Message {
    fn from(value: Names) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Names> for RawMessage {
    fn from(value: Names) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Names {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Names, ClientMessageError> {
        todo!()
    }
}
