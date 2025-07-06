use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Quit {
    reason: Option<FinalParam>,
}

impl Quit {
    pub fn new() -> Quit {
        Quit { reason: None }
    }

    pub fn new_with_reason(reason: FinalParam) -> Quit {
        Quit {
            reason: Some(reason),
        }
    }

    pub fn reason(&self) -> Option<&FinalParam> {
        self.reason.as_ref()
    }

    pub fn into_reason(self) -> Option<FinalParam> {
        self.reason
    }
}

impl ClientMessageParts for Quit {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Quit,
            ParameterList::builder().maybe_with_final(self.reason),
        )
    }
}

impl ToIrcLine for Quit {
    fn to_irc_line(&self) -> String {
        let mut s = String::from("QUIT");
        if let Some(ref reason) = self.reason {
            s.push(' ');
            s.push(':');
            s.push_str(reason.as_str());
        }
        s
    }
}

impl From<Quit> for Message {
    fn from(value: Quit) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Quit> for RawMessage {
    fn from(value: Quit) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Quit {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Quit, ClientMessageError> {
        let (reason,) = params.try_into()?;
        Ok(Quit { reason })
    }
}
