use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::DisplayMaybeFinal;
use crate::{FinalParam, Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Version {
    target: Option<FinalParam>,
}

impl Version {
    pub fn new() -> Version {
        Version { target: None }
    }

    pub fn new_with_target(target: FinalParam) -> Version {
        Version {
            target: Some(target),
        }
    }

    pub fn target(&self) -> Option<&FinalParam> {
        self.target.as_ref()
    }

    pub fn into_target(self) -> Option<FinalParam> {
        self.target
    }
}

impl ClientMessageParts for Version {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Version,
            ParameterList::builder().maybe_with_final(self.target),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("VERSION{}", DisplayMaybeFinal(self.target.as_ref()))
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
