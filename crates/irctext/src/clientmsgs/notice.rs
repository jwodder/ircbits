use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Notice;

impl ClientMessageParts for Notice {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Notice {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Notice> for Message {
    fn from(value: Notice) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Notice> for RawMessage {
    fn from(value: Notice) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Notice {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Notice, ClientMessageError> {
        todo!()
    }
}
