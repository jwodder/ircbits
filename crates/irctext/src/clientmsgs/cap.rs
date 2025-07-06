use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cap;

impl ClientMessageParts for Cap {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Cap {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Cap> for Message {
    fn from(value: Cap) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Cap> for RawMessage {
    fn from(value: Cap) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Cap {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Cap, ClientMessageError> {
        todo!()
    }
}
