use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Join;

impl ClientMessageParts for Join {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Join {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Join> for Message {
    fn from(value: Join) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Join> for RawMessage {
    fn from(value: Join) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Join {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Join, ClientMessageError> {
        todo!()
    }
}
