use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Kill;

impl ClientMessageParts for Kill {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Kill {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Kill> for Message {
    fn from(value: Kill) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Kill> for RawMessage {
    fn from(value: Kill) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Kill {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Kill, ClientMessageError> {
        todo!()
    }
}
