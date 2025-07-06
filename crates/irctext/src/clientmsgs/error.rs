use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error;

impl ClientMessageParts for Error {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Error {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Error> for Message {
    fn from(value: Error) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Error> for RawMessage {
    fn from(value: Error) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Error {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Error, ClientMessageError> {
        todo!()
    }
}
