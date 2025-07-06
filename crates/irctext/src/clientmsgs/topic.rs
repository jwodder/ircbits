use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Topic;

impl ClientMessageParts for Topic {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Topic {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Topic> for Message {
    fn from(value: Topic) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Topic> for RawMessage {
    fn from(value: Topic) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Topic {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Topic, ClientMessageError> {
        todo!()
    }
}
