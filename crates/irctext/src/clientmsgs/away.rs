use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Away;

impl ClientMessageParts for Away {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Away {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Away> for Message {
    fn from(value: Away) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Away> for RawMessage {
    fn from(value: Away) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Away {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Away, ClientMessageError> {
        todo!()
    }
}
