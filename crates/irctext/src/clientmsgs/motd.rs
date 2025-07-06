use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Motd {
    target: Option<FinalParam>,
}

impl Motd {
    pub fn new() -> Motd {
        Motd { target: None }
    }

    pub fn new_with_target(target: FinalParam) -> Motd {
        Motd {
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

impl ClientMessageParts for Motd {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Motd,
            ParameterList::builder().maybe_with_final(self.target),
        )
    }
}

impl ToIrcLine for Motd {
    fn to_irc_line(&self) -> String {
        let mut s = String::from("MOTD");
        if let Some(ref target) = self.target {
            s.push(' ');
            s.push(':');
            s.push_str(target.as_str());
        }
        s
    }
}

impl From<Motd> for Message {
    fn from(value: Motd) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Motd> for RawMessage {
    fn from(value: Motd) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Motd {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Motd, ClientMessageError> {
        let (target,) = params.try_into()?;
        Ok(Motd { target })
    }
}
