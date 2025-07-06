use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Admin;

impl ClientMessageParts for Admin {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Admin {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Admin> for Message {
    fn from(value: Admin) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Admin> for RawMessage {
    fn from(value: Admin) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Admin {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Admin, ClientMessageError> {
        todo!()
    }
}
