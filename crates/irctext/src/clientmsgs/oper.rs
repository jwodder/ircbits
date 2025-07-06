use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, MedialParam, Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Oper {
    name: MedialParam,
    password: FinalParam,
}

impl Oper {
    pub fn new(name: MedialParam, password: FinalParam) -> Oper {
        Oper { name, password }
    }

    pub fn name(&self) -> &MedialParam {
        &self.name
    }

    pub fn password(&self) -> &FinalParam {
        &self.password
    }
}

impl ClientMessageParts for Oper {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Oper,
            ParameterList::builder()
                .with_medial(self.name)
                .with_final(self.password),
        )
    }
}

impl ToIrcLine for Oper {
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
