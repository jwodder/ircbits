use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Version(Option<FinalParam>);

impl Version {
    pub fn new() -> Version {
        Version(None)
    }

    pub fn new_with_target(target: FinalParam) -> Version {
        Version(Some(target))
    }

    pub fn target(&self) -> Option<&FinalParam> {
        self.0.as_ref()
    }

    pub fn into_target(self) -> Option<FinalParam> {
        self.0
    }
}

impl ClientMessageParts for Version {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Version,
            ParameterList::builder().maybe_with_final(self.0),
        )
    }
}

impl ToIrcLine for Version {
    fn to_irc_line(&self) -> String {
        let mut s = String::from("VERSION");
        if let Some(ref target) = self.0 {
            s.push(' ');
            s.push(':');
            s.push_str(target.as_str());
        }
        s
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
        if params.is_empty() {
            Ok(Version::new())
        } else {
            let (p,) = params.try_into()?;
            Ok(Version::new_with_target(p))
        }
    }
}
