use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mode;

impl ClientMessageParts for Mode {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Mode {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Mode> for Message {
    fn from(value: Mode) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Mode> for RawMessage {
    fn from(value: Mode) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Mode {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Mode, ClientMessageError> {
        todo!()
    }
}
