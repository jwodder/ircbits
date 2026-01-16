use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::DisplayMaybeTrailing;
use crate::{Message, ParameterList, RawMessage, TrailingParam, Verb};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Help {
    subject: Option<TrailingParam>,
}

impl Help {
    pub fn new() -> Help {
        Help { subject: None }
    }

    pub fn new_with_subject(subject: TrailingParam) -> Help {
        Help {
            subject: Some(subject),
        }
    }

    pub fn subject(&self) -> Option<&TrailingParam> {
        self.subject.as_ref()
    }

    pub fn into_subject(self) -> Option<TrailingParam> {
        self.subject
    }
}

impl ClientMessageParts for Help {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Help,
            ParameterList::builder().maybe_with_trailing(self.subject),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("HELP{}", DisplayMaybeTrailing(self.subject.as_ref()))
    }
}

impl From<Help> for Message {
    fn from(value: Help) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Help> for RawMessage {
    fn from(value: Help) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Help {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Help, ClientMessageError> {
        let (subject,) = params.try_into()?;
        Ok(Help { subject })
    }
}
