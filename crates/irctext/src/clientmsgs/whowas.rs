use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::DisplayMaybeFinal;
use crate::{FinalParam, Message, Nickname, ParameterList, RawMessage, Verb};
use std::num::NonZeroUsize;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoWas {
    nick: Nickname,
    count: Option<NonZeroUsize>,
}

impl WhoWas {
    pub fn new(nick: Nickname) -> WhoWas {
        WhoWas { nick, count: None }
    }

    pub fn new_with_count(nick: Nickname, count: NonZeroUsize) -> WhoWas {
        WhoWas {
            nick,
            count: Some(count),
        }
    }

    pub fn nick(&self) -> &Nickname {
        &self.nick
    }

    pub fn count(&self) -> Option<NonZeroUsize> {
        self.count
    }
}

impl ClientMessageParts for WhoWas {
    fn into_parts(self) -> (Verb, ParameterList) {
        let params = ParameterList::builder()
            .with_medial(self.nick)
            .maybe_with_final(self.count.map(|c| {
                let Ok(param) = FinalParam::try_from(c.to_string()) else {
                    unreachable!("A stringified integer should be a valid FinalParam");
                };
                param
            }));
        (Verb::WhoWas, params)
    }

    fn to_irc_line(&self) -> String {
        format!("WHOWAS {}{}", self.nick, DisplayMaybeFinal(self.count))
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
        let nick = match p1.as_str().parse::<Nickname>() {
            Ok(nick) => nick,
            Err(source) => {
                return Err(ClientMessageError::ParseParam {
                    index: 0,
                    raw: p1.into_inner(),
                    source: Box::new(source),
                })
            }
        };
        let count = if let Some(p) = p2 {
            match p.as_str().parse::<NonZeroUsize>() {
                Ok(count) => Some(count),
                Err(source) => {
                    return Err(ClientMessageError::ParseParam {
                        index: 1,
                        raw: p.into_inner(),
                        source: Box::new(source),
                    })
                }
            }
        } else {
            None
        };
        Ok(WhoWas { nick, count })
    }
}
