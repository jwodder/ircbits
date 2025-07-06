use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivMsg;

impl ClientMessageParts for PrivMsg {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for PrivMsg {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<PrivMsg> for Message {
    fn from(value: PrivMsg) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<PrivMsg> for RawMessage {
    fn from(value: PrivMsg) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for PrivMsg {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<PrivMsg, ClientMessageError> {
        todo!()
    }
}
