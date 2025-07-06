use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIs;

impl ClientMessageParts for WhoIs {
    fn into_parts(self) -> (Verb, ParameterList) {
        todo!()
    }
}

impl ToIrcLine for WhoIs {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

impl From<WhoIs> for Message {
    fn from(value: WhoIs) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<WhoIs> for RawMessage {
    fn from(value: WhoIs) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for WhoIs {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<WhoIs, ClientMessageError> {
        todo!()
    }
}
