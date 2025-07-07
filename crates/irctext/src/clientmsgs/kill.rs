use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, Message, Nickname, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Kill {
    nickname: Nickname,
    comment: FinalParam,
}

impl Kill {
    pub fn new(nickname: Nickname, comment: FinalParam) -> Kill {
        Kill { nickname, comment }
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn comment(&self) -> &FinalParam {
        &self.comment
    }
}

impl ClientMessageParts for Kill {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Kill,
            ParameterList::builder()
                .with_medial(self.nickname)
                .with_final(self.comment),
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
        let (p1, comment): (_, FinalParam) = params.try_into()?;
        match p1.as_str().parse::<Nickname>() {
            Ok(nickname) => Ok(Kill { nickname, comment }),
            Err(source) => Err(ClientMessageError::ParseParam {
                index: 0,
                raw: p1.into_inner(),
                source: Box::new(source),
            }),
        }
    }
}
