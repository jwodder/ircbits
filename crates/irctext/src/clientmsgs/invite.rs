use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Invite;

impl ClientMessageParts for Invite {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for Invite {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<Invite> for Message {
    fn from(value: Invite) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Invite> for RawMessage {
    fn from(value: Invite) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Invite {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Invite, ClientMessageError> {
        todo!()
    }
}
