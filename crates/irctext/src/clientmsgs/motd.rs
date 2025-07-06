use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Motd;

impl ClientMessageParts for Motd {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Motd {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Motd> for Message {
    fn from(value: Motd) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Motd> for RawMessage {
    fn from(value: Motd) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Motd {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Motd, ClientMessageError> {
        todo!()
    }
}
