use super::{ClientMessage, ClientMessageError, ClientMessageParts, Pong};
use crate::{Message, ParameterList, RawMessage, TrailingParam, Verb};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Ping {
    token: TrailingParam,
}

impl Ping {
    pub fn new(token: TrailingParam) -> Ping {
        Ping { token }
    }

    pub fn token(&self) -> &TrailingParam {
        &self.token
    }

    pub fn into_token(self) -> TrailingParam {
        self.token
    }

    pub fn to_pong(&self) -> Pong {
        Pong::new(self.token.clone())
    }

    pub fn into_pong(self) -> Pong {
        Pong::new(self.token)
    }
}

impl ClientMessageParts for Ping {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Ping,
            ParameterList::builder().with_trailing(self.token),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("PING :{}", self.token)
    }
}

impl From<Ping> for Message {
    fn from(value: Ping) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Ping> for RawMessage {
    fn from(value: Ping) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Ping {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Ping, ClientMessageError> {
        let (token,) = params.try_into()?;
        Ok(Ping { token })
    }
}
