use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Rehash;

impl ClientMessageParts for Rehash {
    fn into_parts(self) -> (Verb, ParameterList) {
        (Verb::Rehash, ParameterList::default())
    }
}

impl ToIrcLine for Rehash {
    fn to_irc_line(&self) -> String {
        String::from("REHASH")
    }
}

impl From<Rehash> for Message {
    fn from(value: Rehash) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Rehash> for RawMessage {
    fn from(value: Rehash) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Rehash {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Rehash, ClientMessageError> {
        let () = params.try_into()?;
        Ok(Rehash)
    }
}
