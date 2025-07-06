use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Lusers;

impl ClientMessageParts for Lusers {
    fn into_parts(self) -> (Verb, ParameterList) {
        (Verb::Lusers, ParameterList::default())
    }
}

impl ToIrcLine for Lusers {
    fn to_irc_line(&self) -> String {
        String::from("LUSERS")
    }
}

impl From<Lusers> for Message {
    fn from(value: Lusers) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Lusers> for RawMessage {
    fn from(value: Lusers) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Lusers {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Lusers, ClientMessageError> {
        let () = params.try_into()?;
        Ok(Lusers)
    }
}
