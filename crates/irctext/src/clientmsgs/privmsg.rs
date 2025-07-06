use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::join_with_commas;
use crate::{
    Channel, ChannelError, FinalParam, MedialParam, Message, Nickname, NicknameError,
    ParameterList, RawMessage, ToIrcLine, Verb,
};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivMsg {
    targets: Vec<PrivMsgTarget>,
    text: FinalParam,
}

impl PrivMsg {
    pub fn new<T: Into<PrivMsgTarget>>(target: T, text: FinalParam) -> PrivMsg {
        PrivMsg {
            targets: vec![target.into()],
            text,
        }
    }

    pub fn new_to_many<I, T>(targets: I, text: FinalParam) -> Option<PrivMsg>
    where
        I: IntoIterator<Item = T>,
        T: Into<PrivMsgTarget>,
    {
        let targets = targets.into_iter().map(Into::into).collect::<Vec<_>>();
        if targets.is_empty() {
            None
        } else {
            Some(PrivMsg { targets, text })
        }
    }

    pub fn targets(&self) -> &[PrivMsgTarget] {
        &self.targets
    }

    pub fn text(&self) -> &FinalParam {
        &self.text
    }

    fn targets_param(&self) -> MedialParam {
        assert!(
            !self.targets.is_empty(),
            "PrivMsg.targets should always be nonempty"
        );
        let s = join_with_commas(&self.targets);
        MedialParam::try_from(s)
            .expect("comma-separated channels and/or nicknames should be a valid MedialParam")
    }
}

impl ClientMessageParts for PrivMsg {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::PrivMsg,
            ParameterList::builder()
                .with_medial(self.targets_param())
                .with_final(self.text),
        )
    }
}

impl ToIrcLine for PrivMsg {
    fn to_irc_line(&self) -> String {
        format!("PRIVMSG {} :{}", self.targets_param(), self.text)
    }
}

impl From<PrivMsg> for Message {
    fn from(value: PrivMsg) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<PrivMsg> for RawMessage {
    fn from(value: PrivMsg) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for PrivMsg {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<PrivMsg, ClientMessageError> {
        let (p1, text): (_, FinalParam) = params.try_into()?;
        match p1
            .as_str()
            .split(',')
            .map(str::parse::<PrivMsgTarget>)
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(targets) => Ok(PrivMsg { targets, text }),
            Err(source) => Err(ClientMessageError::ParseParam {
                index: 0,
                raw: p1.into_inner(),
                source: Box::new(source),
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrivMsgTarget {
    Channel(Channel),
    User(Nickname),
}

impl std::str::FromStr for PrivMsgTarget {
    type Err = PrivMsgTargetError;

    fn from_str(s: &str) -> Result<PrivMsgTarget, PrivMsgTargetError> {
        // TODO: Improve this!
        if s.starts_with(['#', '&']) {
            let channel = s.parse::<Channel>()?;
            Ok(PrivMsgTarget::Channel(channel))
        } else {
            let nickname = s.parse::<Nickname>()?;
            Ok(PrivMsgTarget::User(nickname))
        }
    }
}

impl AsRef<str> for PrivMsgTarget {
    fn as_ref(&self) -> &str {
        match self {
            PrivMsgTarget::Channel(chan) => chan.as_ref(),
            PrivMsgTarget::User(nick) => nick.as_ref(),
        }
    }
}

impl From<Channel> for PrivMsgTarget {
    fn from(value: Channel) -> PrivMsgTarget {
        PrivMsgTarget::Channel(value)
    }
}

impl From<Nickname> for PrivMsgTarget {
    fn from(value: Nickname) -> PrivMsgTarget {
        PrivMsgTarget::User(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum PrivMsgTargetError {
    #[error(transparent)]
    Channel(#[from] ChannelError),
    #[error(transparent)]
    Nickname(#[from] NicknameError),
}
