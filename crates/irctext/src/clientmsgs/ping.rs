use super::{ClientMessage, ClientMessageError, ClientMessageParts, Pong};
use crate::{FinalParam, Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ping(FinalParam);

impl Ping {
    pub fn new(token: FinalParam) -> Ping {
        Ping(token)
    }

    pub fn token(&self) -> &FinalParam {
        &self.0
    }

    pub fn into_token(self) -> FinalParam {
        self.0
    }

    pub fn to_pong(&self) -> Pong {
        Pong::new(self.0.clone())
    }

    pub fn into_pong(self) -> Pong {
        Pong::new(self.0)
    }
}

impl ClientMessageParts for Ping {
    fn into_parts(self) -> (Verb, ParameterList) {
        (Verb::Ping, ParameterList::builder().with_final(self.0))
    }
}

impl ToIrcLine for Ping {
    fn to_irc_line(&self) -> String {
        format!("PING :{}", self.0)
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
        let (p,) = params.try_into()?;
        Ok(Ping(p))
    }
}
