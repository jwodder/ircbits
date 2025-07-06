use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Help;

impl ClientMessageParts for Help {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Help {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Help> for Message {
    fn from(value: Help) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Help> for RawMessage {
    fn from(value: Help) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Help {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Help, ClientMessageError> {
        todo!()
    }
}
