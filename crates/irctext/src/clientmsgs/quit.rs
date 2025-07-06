use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Quit(Option<FinalParam>);

impl Quit {
    pub fn new() -> Quit {
        Quit(None)
    }

    pub fn new_with_reason(reason: FinalParam) -> Quit {
        Quit(Some(reason))
    }

    pub fn reason(&self) -> Option<&FinalParam> {
        self.0.as_ref()
    }

    pub fn into_reason(self) -> Option<FinalParam> {
        self.0
    }
}

impl ClientMessageParts for Quit {
    fn into_parts(self) -> (Verb, ParameterList) {
        let builder = ParameterList::builder();
        let params = if let Some(reason) = self.0 {
            builder.with_final(reason)
        } else {
            builder.finish()
        };
        (Verb::Quit, params)
    }
}

impl ToIrcLine for Quit {
    fn to_irc_line(&self) -> String {
        let mut s = String::from("QUIT");
        if let Some(ref reason) = self.0 {
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
        if params.is_empty() {
            Ok(Quit::new())
        } else {
            let (p,) = params.try_into()?;
            Ok(Quit::new_with_reason(p))
        }
    }
}
