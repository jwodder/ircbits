use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Kick;

impl ClientMessageParts for Kick {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Kick {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Kick> for Message {
    fn from(value: Kick) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Kick> for RawMessage {
    fn from(value: Kick) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Kick {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Kick, ClientMessageError> {
        todo!()
    }
}
