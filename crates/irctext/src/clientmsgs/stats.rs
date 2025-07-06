use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stats;

impl ClientMessageParts for Stats {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<Stats> for Message {
    fn from(value: Stats) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Stats> for RawMessage {
    fn from(value: Stats) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Stats {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Stats, ClientMessageError> {
        todo!()
    }
}
