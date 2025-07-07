use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, MedialParam, Message, Nickname, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIs {
    target: Option<MedialParam>,
    nick: Nickname,
}

impl WhoIs {
    pub fn new(nick: Nickname) -> WhoIs {
        WhoIs { target: None, nick }
    }

    pub fn new_with_target(nick: Nickname, target: MedialParam) -> WhoIs {
        WhoIs {
            target: Some(target),
            nick,
        }
    }

    pub fn target(&self) -> Option<&MedialParam> {
        self.target.as_ref()
    }

    pub fn nick(&self) -> &Nickname {
        &self.nick
    }
}

impl ClientMessageParts for WhoIs {
    fn into_parts(self) -> (Verb, ParameterList) {
        let mut builder = ParameterList::builder();
        if let Some(target) = self.target {
            builder.push_medial(target);
        }
        let params = builder.with_medial(self.nick).finish();
        (Verb::WhoIs, params)
    }

    fn to_irc_line(&self) -> String {
        if let Some(ref target) = self.target {
            format!("WHOIS {target} {}", self.nick)
        } else {
            format!("WHOIS {}", self.nick)
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
        let (target, rawnick, index) = if let Some(p2) = p2 {
            (Some(p1), p2.into_inner(), 1)
        } else {
            (None, p1.into_inner(), 0)
        };
        match rawnick.parse::<Nickname>() {
            Ok(nick) => Ok(WhoIs { target, nick }),
            Err(source) => Err(ClientMessageError::ParseParam {
                index,
                raw: rawnick,
                source: Box::new(source),
            }),
        }
    }
}
