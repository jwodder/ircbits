use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, ParameterList, ParameterListSizeError, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Info;

impl ClientMessageParts for Info {
    fn into_parts(self) -> (Verb, ParameterList) {
        (Verb::Info, ParameterList::default())
    }
}

impl ToIrcLine for Info {
    fn to_irc_line(&self) -> String {
        String::from("INFO")
    }
}

impl From<Info> for Message {
    fn from(value: Info) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Info> for RawMessage {
    fn from(value: Info) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Info {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Info, ClientMessageError> {
        let () = params.try_into()?;
        Ok(Info)
    }
}
