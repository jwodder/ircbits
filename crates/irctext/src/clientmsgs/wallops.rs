use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Wallops;

impl ClientMessageParts for Wallops {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Wallops {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Wallops> for Message {
    fn from(value: Wallops) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Wallops> for RawMessage {
    fn from(value: Wallops) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Wallops {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Wallops, ClientMessageError> {
        todo!()
    }
}
