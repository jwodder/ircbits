use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pong {
    token: FinalParam,
}

impl Pong {
    pub fn new(token: FinalParam) -> Pong {
        Pong { token }
    }

    pub fn token(&self) -> &FinalParam {
        &self.token
    }

    pub fn into_token(self) -> FinalParam {
        self.token
    }
}

impl ClientMessageParts for Pong {
    fn into_parts(self) -> (Verb, ParameterList) {
        (Verb::Pong, ParameterList::builder().with_final(self.token))
    }
}

impl ToIrcLine for Pong {
    fn to_irc_line(&self) -> String {
        format!("PONG :{}", self.token)
    }
}

impl From<Pong> for Message {
    fn from(value: Pong) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Pong> for RawMessage {
    fn from(value: Pong) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Pong {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Pong, ClientMessageError> {
        let (token,) = params.try_into()?;
        Ok(Pong { token })
    }
}
