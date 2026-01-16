use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::DisplayMaybeTrailing;
use crate::{Message, ParameterList, RawMessage, TrailingParam, Verb};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Version {
    target: Option<TrailingParam>,
}

impl Version {
    pub fn new() -> Version {
        Version { target: None }
    }

    pub fn new_with_target(target: TrailingParam) -> Version {
        Version {
            target: Some(target),
        }
    }

    pub fn target(&self) -> Option<&TrailingParam> {
        self.target.as_ref()
    }

    pub fn into_target(self) -> Option<TrailingParam> {
        self.target
    }
}

impl ClientMessageParts for Version {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Version,
            ParameterList::builder().maybe_with_trailing(self.target),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("VERSION{}", DisplayMaybeTrailing(self.target.as_ref()))
    }
}

impl From<Version> for Message {
    fn from(value: Version) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Version> for RawMessage {
    fn from(value: Version) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Version {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Version, ClientMessageError> {
        let (target,) = params.try_into()?;
        Ok(Version { target })
    }
}
