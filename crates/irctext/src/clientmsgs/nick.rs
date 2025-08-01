use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::types::Nickname;
use crate::{FinalParam, Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Nick {
    nickname: Nickname,
}

impl Nick {
    pub fn new(nickname: Nickname) -> Nick {
        Nick { nickname }
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn into_nickname(self) -> Nickname {
        self.nickname
    }
}

impl ClientMessageParts for Nick {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Nick,
            ParameterList::builder().with_medial(self.nickname).finish(),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("NICK {}", self.nickname)
    }
}

impl From<Nick> for Message {
    fn from(value: Nick) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Nick> for RawMessage {
    fn from(value: Nick) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Nick {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Nick, ClientMessageError> {
        let (p,): (FinalParam,) = params.try_into()?;
        let nickname = Nickname::try_from(p.into_inner())?;
        Ok(Nick { nickname })
    }
}
