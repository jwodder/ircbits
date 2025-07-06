use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pass;

impl ClientMessageParts for Pass {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Pass {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Pass> for Message {
    fn from(value: Pass) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Pass> for RawMessage {
    fn from(value: Pass) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Pass {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Pass, ClientMessageError> {
        todo!()
    }
}
