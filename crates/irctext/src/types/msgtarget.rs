use crate::types::{
    Channel, Nickname, ParseChannelError, ParseNicknameError, channel::channel_prefixed,
};
use crate::{FinalParam, MedialParam, TryFromStringError};
use std::fmt;
use thiserror::Error;

/// The target of a `PRIVMSG` or `NOTICE` message
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MsgTarget {
    Channel(Channel),
    Nick(Nickname),
    Star,
}

impl MsgTarget {
    pub fn is_channel(&self) -> bool {
        matches!(self, MsgTarget::Channel(_))
    }

    pub fn is_nick(&self) -> bool {
        matches!(self, MsgTarget::Nick(_))
    }

    pub fn is_star(&self) -> bool {
        matches!(self, MsgTarget::Star)
    }

    pub fn as_str(&self) -> &str {
        match self {
            MsgTarget::Channel(chan) => chan.as_str(),
            MsgTarget::Nick(nick) => nick.as_str(),
            MsgTarget::Star => "*",
        }
    }
}

impl fmt::Display for MsgTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for MsgTarget {
    type Err = ParseMsgTargetError;

    fn from_str(s: &str) -> Result<MsgTarget, ParseMsgTargetError> {
        if s == "*" {
            Ok(MsgTarget::Star)
        } else if channel_prefixed(s) {
            let channel = s.parse::<Channel>()?;
            Ok(MsgTarget::Channel(channel))
        } else {
            let nickname = s.parse::<Nickname>()?;
            Ok(MsgTarget::Nick(nickname))
        }
    }
}

impl TryFrom<String> for MsgTarget {
    type Error = TryFromStringError<ParseMsgTargetError>;

    fn try_from(value: String) -> Result<MsgTarget, TryFromStringError<ParseMsgTargetError>> {
        if value == "*" {
            Ok(MsgTarget::Star)
        } else if channel_prefixed(&value) {
            match Channel::try_from(value) {
                Ok(channel) => Ok(MsgTarget::Channel(channel)),
                Err(TryFromStringError { inner, string }) => Err(TryFromStringError {
                    inner: ParseMsgTargetError::Channel(inner),
                    string,
                }),
            }
        } else {
            match Nickname::try_from(value) {
                Ok(nickname) => Ok(MsgTarget::Nick(nickname)),
                Err(TryFromStringError { inner, string }) => Err(TryFromStringError {
                    inner: ParseMsgTargetError::Nickname(inner),
                    string,
                }),
            }
        }
    }
}

impl AsRef<str> for MsgTarget {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<Channel> for MsgTarget {
    fn from(value: Channel) -> MsgTarget {
        MsgTarget::Channel(value)
    }
}

impl From<Nickname> for MsgTarget {
    fn from(value: Nickname) -> MsgTarget {
        MsgTarget::Nick(value)
    }
}

impl From<MsgTarget> for String {
    fn from(value: MsgTarget) -> String {
        match value {
            MsgTarget::Channel(chan) => chan.into(),
            MsgTarget::Nick(nick) => nick.into(),
            MsgTarget::Star => String::from("*"),
        }
    }
}

impl From<MsgTarget> for MedialParam {
    fn from(value: MsgTarget) -> MedialParam {
        MedialParam::try_from(String::from(value))
            .expect("Message target should be valid MedialParam")
    }
}

impl From<MsgTarget> for FinalParam {
    fn from(value: MsgTarget) -> FinalParam {
        FinalParam::from(MedialParam::from(value))
    }
}

impl PartialEq<String> for MsgTarget {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<str> for MsgTarget {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<'a> PartialEq<&'a str> for MsgTarget {
    fn eq(&self, other: &&'a str) -> bool {
        &self.as_str() == other
    }
}

impl PartialEq<Channel> for MsgTarget {
    fn eq(&self, other: &Channel) -> bool {
        matches!(self, MsgTarget::Channel(chan) if chan == other)
    }
}

impl PartialEq<Nickname> for MsgTarget {
    fn eq(&self, other: &Nickname) -> bool {
        matches!(self, MsgTarget::Nick(nick) if nick == other)
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseMsgTargetError {
    #[error(transparent)]
    Channel(#[from] ParseChannelError),
    #[error(transparent)]
    Nickname(#[from] ParseNicknameError),
}
