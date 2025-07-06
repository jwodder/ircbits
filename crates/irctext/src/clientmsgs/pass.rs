use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, Message, ParameterList, RawMessage, ToIrcLine, Verb};

pub type Password = FinalParam;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pass(Password);

impl Pass {
    pub fn new(password: Password) -> Pass {
        Pass(password)
    }

    pub fn password(&self) -> &Password {
        &self.0
    }

    pub fn into_password(self) -> Password {
        self.0
    }
}

impl ClientMessageParts for Pass {
    fn into_parts(self) -> (Verb, ParameterList) {
        (Verb::Pass, ParameterList::builder().with_final(self.0))
    }
}

impl ToIrcLine for Pass {
    fn to_irc_line(&self) -> String {
        format!("PASS :{}", self.0)
    }
}

impl From<Pass> for Message {
    fn from(value: Pass) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Pass> for RawMessage {
    fn from(value: Pass) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Pass {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Pass, ClientMessageError> {
        let (p,) = params.try_into()?;
        Ok(Pass(p))
    }
}
