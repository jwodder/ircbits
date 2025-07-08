use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::types::Nickname;
use crate::{FinalParam, MedialParam, Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIs {
    target: Option<MedialParam>,
    nickname: Nickname,
}

impl WhoIs {
    pub fn new(nickname: Nickname) -> WhoIs {
        WhoIs {
            target: None,
            nickname,
        }
    }

    pub fn new_with_target(nickname: Nickname, target: MedialParam) -> WhoIs {
        WhoIs {
            target: Some(target),
            nickname,
        }
    }

    pub fn target(&self) -> Option<&MedialParam> {
        self.target.as_ref()
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }
}

impl ClientMessageParts for WhoIs {
    fn into_parts(self) -> (Verb, ParameterList) {
        let mut builder = ParameterList::builder();
        if let Some(target) = self.target {
            builder.push_medial(target);
        }
        let params = builder.with_medial(self.nickname).finish();
        (Verb::WhoIs, params)
    }

    fn to_irc_line(&self) -> String {
        if let Some(ref target) = self.target {
            format!("WHOIS {target} {}", self.nickname)
        } else {
            format!("WHOIS {}", self.nickname)
        }
    }
}

impl From<WhoIs> for Message {
    fn from(value: WhoIs) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<WhoIs> for RawMessage {
    fn from(value: WhoIs) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for WhoIs {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<WhoIs, ClientMessageError> {
        let (p1, p2): (_, Option<FinalParam>) = params.try_into()?;
        let (target, rawnick) = if let Some(p2) = p2 {
            (Some(p1), p2.into_inner())
        } else {
            (None, p1.into_inner())
        };
        let nickname = Nickname::try_from(rawnick)?;
        Ok(WhoIs { target, nickname })
    }
}
