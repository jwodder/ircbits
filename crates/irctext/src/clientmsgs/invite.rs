use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Channel, FinalParam, Message, Nickname, ParameterList, RawMessage, Verb};

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
        let nickname = Nickname::try_from(p1.into_inner())?;
        let channel = Channel::try_from(p2.into_inner())?;
        Ok(Invite { nickname, channel })
    }
}
