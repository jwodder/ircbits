use crate::types::{
    channel::channel_prefixed, Channel, Nickname, ParseChannelError, ParseNicknameError,
};
use crate::{FinalParam, MedialParam, TryFromStringError};
use std::fmt;
use thiserror::Error;

/// The target of a `MODE` message
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModeTarget {
    Channel(Channel),
    Nick(Nickname),
}

impl ModeTarget {
    pub fn is_channel(&self) -> bool {
        matches!(self, ModeTarget::Channel(_))
    }

    pub fn is_nick(&self) -> bool {
        matches!(self, ModeTarget::Nick(_))
    }

    pub fn as_str(&self) -> &str {
        match self {
            ModeTarget::Channel(chan) => chan.as_str(),
            ModeTarget::Nick(nick) => nick.as_str(),
        }
    }
}

impl fmt::Display for ModeTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModeTarget::Channel(channel) => write!(f, "{channel}"),
            ModeTarget::Nick(nick) => write!(f, "{nick}"),
        }
    }
}

impl std::str::FromStr for ModeTarget {
    type Err = ParseModeTargetError;

    fn from_str(s: &str) -> Result<ModeTarget, ParseModeTargetError> {
        if channel_prefixed(s) {
            let channel = s.parse::<Channel>()?;
            Ok(ModeTarget::Channel(channel))
        } else {
            let nickname = s.parse::<Nickname>()?;
            Ok(ModeTarget::Nick(nickname))
        }
    }
}

impl TryFrom<String> for ModeTarget {
    type Error = TryFromStringError<ParseModeTargetError>;

    fn try_from(value: String) -> Result<ModeTarget, TryFromStringError<ParseModeTargetError>> {
        if channel_prefixed(&value) {
            match Channel::try_from(value) {
                Ok(channel) => Ok(ModeTarget::Channel(channel)),
                Err(TryFromStringError { inner, string }) => Err(TryFromStringError {
                    inner: ParseModeTargetError::Channel(inner),
                    string,
                }),
            }
        } else {
            match Nickname::try_from(value) {
                Ok(nickname) => Ok(ModeTarget::Nick(nickname)),
                Err(TryFromStringError { inner, string }) => Err(TryFromStringError {
                    inner: ParseModeTargetError::Nickname(inner),
                    string,
                }),
            }
        }
    }
}

impl AsRef<str> for ModeTarget {
    fn as_ref(&self) -> &str {
        match self {
            ModeTarget::Channel(chan) => chan.as_ref(),
            ModeTarget::Nick(nick) => nick.as_ref(),
        }
    }
}

impl From<Channel> for ModeTarget {
    fn from(value: Channel) -> ModeTarget {
        ModeTarget::Channel(value)
    }
}

impl From<Nickname> for ModeTarget {
    fn from(value: Nickname) -> ModeTarget {
        ModeTarget::Nick(value)
    }
}

impl From<ModeTarget> for String {
    fn from(value: ModeTarget) -> String {
        match value {
            ModeTarget::Channel(chan) => chan.into_inner(),
            ModeTarget::Nick(nick) => nick.into_inner(),
        }
    }
}

impl From<ModeTarget> for MedialParam {
    fn from(value: ModeTarget) -> MedialParam {
        MedialParam::try_from(String::from(value)).expect("Mode target should be valid MedialParam")
    }
}

impl From<ModeTarget> for FinalParam {
    fn from(value: ModeTarget) -> FinalParam {
        FinalParam::from(MedialParam::from(value))
    }
}

impl PartialEq<String> for ModeTarget {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<str> for ModeTarget {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<'a> PartialEq<&'a str> for ModeTarget {
    fn eq(&self, other: &&'a str) -> bool {
        &self.as_str() == other
    }
}

impl PartialEq<Channel> for ModeTarget {
    fn eq(&self, other: &Channel) -> bool {
        matches!(self, ModeTarget::Channel(chan) if chan == other)
    }
}

impl PartialEq<Nickname> for ModeTarget {
    fn eq(&self, other: &Nickname) -> bool {
        matches!(self, ModeTarget::Nick(nick) if nick == other)
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseModeTargetError {
    #[error(transparent)]
    Channel(#[from] ParseChannelError),
    #[error(transparent)]
    Nickname(#[from] ParseNicknameError),
}
