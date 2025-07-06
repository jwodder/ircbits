use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Time;

impl ClientMessageParts for Time {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Time {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Time> for Message {
    fn from(value: Time) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Time> for RawMessage {
    fn from(value: Time) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Time {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Time, ClientMessageError> {
        todo!()
    }
}
