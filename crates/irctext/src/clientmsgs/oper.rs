use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, MiddleParam, ParameterList, RawMessage, TrailingParam, Verb};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Oper {
    name: MiddleParam,
    password: TrailingParam,
}

impl Oper {
    pub fn new(name: MiddleParam, password: TrailingParam) -> Oper {
        Oper { name, password }
    }

    pub fn name(&self) -> &MiddleParam {
        &self.name
    }

    pub fn password(&self) -> &TrailingParam {
        &self.password
    }
}

impl ClientMessageParts for Oper {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Oper,
            ParameterList::builder()
                .with_middle(self.name)
                .with_trailing(self.password),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("OPER {} :{}", self.name, self.password)
    }
}

impl From<Oper> for Message {
    fn from(value: Oper) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Oper> for RawMessage {
    fn from(value: Oper) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Oper {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Oper, ClientMessageError> {
        let (name, password) = params.try_into()?;
        Ok(Oper { name, password })
    }
}
