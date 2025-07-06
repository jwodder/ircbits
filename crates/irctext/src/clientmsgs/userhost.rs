use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserHost;

impl ClientMessageParts for UserHost {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl From<UserHost> for Message {
    fn from(value: UserHost) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<UserHost> for RawMessage {
    fn from(value: UserHost) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for UserHost {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<UserHost, ClientMessageError> {
        todo!()
    }
}
