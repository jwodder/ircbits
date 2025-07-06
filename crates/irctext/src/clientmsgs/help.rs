use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Help {
    subject: Option<FinalParam>,
}

impl Help {
    pub fn new() -> Help {
        Help { subject: None }
    }

    pub fn new_with_subject(subject: FinalParam) -> Help {
        Help {
            subject: Some(subject),
        }
    }

    pub fn subject(&self) -> Option<&FinalParam> {
        self.subject.as_ref()
    }

    pub fn into_subject(self) -> Option<FinalParam> {
        self.subject
    }
}

impl ClientMessageParts for Help {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Help,
            ParameterList::builder().maybe_with_final(self.subject),
        )
    }
}

impl ToIrcLine for Help {
    fn to_irc_line(&self) -> String {
        let mut s = String::from("HELP");
        if let Some(ref subject) = self.subject {
            s.push(' ');
            s.push(':');
            s.push_str(subject.as_str());
        }
        s
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
        if params.is_empty() {
            Ok(Help::new())
        } else {
            let (p,) = params.try_into()?;
            Ok(Help::new_with_subject(p))
        }
    }
}
