use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Wallops {
    text: FinalParam,
}

impl Wallops {
    pub fn new(text: FinalParam) -> Wallops {
        Wallops { text }
    }

    pub fn text(&self) -> &FinalParam {
        &self.text
    }

    pub fn into_text(self) -> FinalParam {
        self.text
    }
}

impl ClientMessageParts for Wallops {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Wallops,
            ParameterList::builder().with_final(self.text),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("WALLOPS :{}", self.text)
    }
}

impl From<Wallops> for Message {
    fn from(value: Wallops) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Wallops> for RawMessage {
    fn from(value: Wallops) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Wallops {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Wallops, ClientMessageError> {
        let (text,) = params.try_into()?;
        Ok(Wallops { text })
    }
}
