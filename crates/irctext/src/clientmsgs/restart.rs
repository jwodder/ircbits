use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Restart;

impl ClientMessageParts for Restart {
    fn into_parts(self) -> (Verb, ParameterList) {
        (Verb::Restart, ParameterList::default())
    }
}

impl ToIrcLine for Restart {
    fn to_irc_line(&self) -> String {
        String::from("RESTART")
    }
}

impl From<Restart> for Message {
    fn from(value: Restart) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Restart> for RawMessage {
    fn from(value: Restart) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Restart {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Restart, ClientMessageError> {
        let () = params.try_into()?;
        Ok(Restart)
    }
}
