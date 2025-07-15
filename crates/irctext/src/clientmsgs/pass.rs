use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Pass {
    password: FinalParam,
}

impl Pass {
    pub fn new(password: FinalParam) -> Pass {
        Pass { password }
    }

    pub fn password(&self) -> &FinalParam {
        &self.password
    }

    pub fn into_password(self) -> FinalParam {
        self.password
    }
}

impl ClientMessageParts for Pass {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Pass,
            ParameterList::builder().with_final(self.password),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("PASS :{}", self.password)
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
        let (password,) = params.try_into()?;
        Ok(Pass { password })
    }
}
