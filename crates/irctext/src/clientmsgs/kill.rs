use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::types::Nickname;
use crate::{Message, ParameterList, RawMessage, TrailingParam, Verb};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Kill {
    nickname: Nickname,
    comment: TrailingParam,
}

impl Kill {
    pub fn new(nickname: Nickname, comment: TrailingParam) -> Kill {
        Kill { nickname, comment }
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn comment(&self) -> &TrailingParam {
        &self.comment
    }
}

impl ClientMessageParts for Kill {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Kill,
            ParameterList::builder()
                .with_middle(self.nickname)
                .with_trailing(self.comment),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("KILL {} :{}", self.nickname, self.comment)
    }
}

impl From<Kill> for Message {
    fn from(value: Kill) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Kill> for RawMessage {
    fn from(value: Kill) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Kill {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Kill, ClientMessageError> {
        let (p1, comment): (_, TrailingParam) = params.try_into()?;
        let nickname = Nickname::try_from(p1.into_inner())?;
        Ok(Kill { nickname, comment })
    }
}
