use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Channel, FinalParam, Message, Nickname, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Invite {
    nickname: Nickname,
    channel: Channel,
}

impl Invite {
    pub fn new(nickname: Nickname, channel: Channel) -> Invite {
        Invite { nickname, channel }
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }
}

impl ClientMessageParts for Invite {
    fn into_parts(self) -> (Verb, ParameterList) {
        let params = ParameterList::builder()
            .with_medial(self.nickname)
            .with_medial(self.channel)
            .finish();
        (Verb::Invite, params)
    }
}

impl ToIrcLine for Invite {
    fn to_irc_line(&self) -> String {
        format!("INVITE {} {}", self.nickname, self.channel)
    }
}

impl From<Invite> for Message {
    fn from(value: Invite) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Invite> for RawMessage {
    fn from(value: Invite) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Invite {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Invite, ClientMessageError> {
        let (p1, p2): (_, FinalParam) = params.try_into()?;
        let nickname = match p1.as_str().parse::<Nickname>() {
            Ok(n) => n,
            Err(source) => {
                return Err(ClientMessageError::ParseParam {
                    index: 0,
                    raw: p1.into_inner(),
                    source: Box::new(source),
                })
            }
        };
        let channel = match p2.as_str().parse::<Channel>() {
            Ok(ch) => ch,
            Err(source) => {
                return Err(ClientMessageError::ParseParam {
                    index: 1,
                    raw: p2.into_inner(),
                    source: Box::new(source),
                })
            }
        };
        Ok(Invite { nickname, channel })
    }
}
