use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Version;

impl ClientMessageParts for Version {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Version {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Version> for Message {
    fn from(value: Version) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Version> for RawMessage {
    fn from(value: Version) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Version {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Version, ClientMessageError> {
        todo!()
    }
}
