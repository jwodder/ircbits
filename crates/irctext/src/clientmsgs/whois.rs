use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Whois;

impl ClientMessageParts for Whois {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<Whois> for Message {
    fn from(value: Whois) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Whois> for RawMessage {
    fn from(value: Whois) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Whois {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Whois, ClientMessageError> {
        todo!()
    }
}
