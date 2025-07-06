use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User;

impl ClientMessageParts for User {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for User {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<User> for Message {
    fn from(value: User) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<User> for RawMessage {
    fn from(value: User) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for User {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<User, ClientMessageError> {
        todo!()
    }
}
