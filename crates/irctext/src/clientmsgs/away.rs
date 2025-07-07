use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::DisplayMaybeFinal;
use crate::{FinalParam, Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Away {
    text: Option<FinalParam>,
}

impl Away {
    pub fn new(text: FinalParam) -> Away {
        Away { text: Some(text) }
    }

    pub fn new_unaway() -> Away {
        Away { text: None }
    }

    pub fn text(&self) -> Option<&FinalParam> {
        self.text.as_ref()
    }

    pub fn into_text(self) -> Option<FinalParam> {
        self.text
    }
}

impl ClientMessageParts for Away {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Away,
            ParameterList::builder().maybe_with_final(self.text),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("AWAY{}", DisplayMaybeFinal(self.text.as_ref()))
    }
}

impl From<Away> for Message {
    fn from(value: Away) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Away> for RawMessage {
    fn from(value: Away) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Away {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Away, ClientMessageError> {
        let (text,) = params.try_into()?;
        Ok(Away { text })
    }
}
