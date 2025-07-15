use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::types::Nickname;
use crate::util::DisplayMaybeFinal;
use crate::{FinalParam, Message, ParameterList, RawMessage, Verb};
use std::num::NonZeroUsize;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WhoWas {
    nickname: Nickname,
    count: Option<NonZeroUsize>,
}

impl WhoWas {
    pub fn new(nickname: Nickname) -> WhoWas {
        WhoWas {
            nickname,
            count: None,
        }
    }

    pub fn new_with_count(nickname: Nickname, count: NonZeroUsize) -> WhoWas {
        WhoWas {
            nickname,
            count: Some(count),
        }
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn count(&self) -> Option<NonZeroUsize> {
        self.count
    }
}

impl ClientMessageParts for WhoWas {
    fn into_parts(self) -> (Verb, ParameterList) {
        let params = ParameterList::builder()
            .with_medial(self.nickname)
            .maybe_with_final(self.count.map(|c| {
                let Ok(param) = FinalParam::try_from(c.to_string()) else {
                    unreachable!("A stringified integer should be a valid FinalParam");
                };
                param
            }));
        (Verb::WhoWas, params)
    }

    fn to_irc_line(&self) -> String {
        format!("WHOWAS {}{}", self.nickname, DisplayMaybeFinal(self.count))
    }
}

impl From<WhoWas> for Message {
    fn from(value: WhoWas) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<WhoWas> for RawMessage {
    fn from(value: WhoWas) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for WhoWas {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<WhoWas, ClientMessageError> {
        let (p1, p2): (_, Option<FinalParam>) = params.try_into()?;
        let nickname = Nickname::try_from(p1.into_inner())?;
        let count = if let Some(p) = p2 {
            match p.as_str().parse::<NonZeroUsize>() {
                Ok(count) => Some(count),
                Err(inner) => {
                    return Err(ClientMessageError::Int {
                        string: p.into_inner(),
                        inner,
                    });
                }
            }
        } else {
            None
        };
        Ok(WhoWas { nickname, count })
    }
}
