use crate::channel::channel_prefixed;
use crate::{
    Channel, FinalParam, MedialParam, Nickname, ParseChannelError, ParseNicknameError,
    TryFromStringError,
};
use std::fmt;
use thiserror::Error;

/// The target of a `MODE` message
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModeTarget {
    Channel(Channel),
    User(Nickname),
}

impl ModeTarget {
    pub fn into_inner(self) -> String {
        match self {
            ModeTarget::Channel(channel) => channel.into_inner(),
            ModeTarget::User(nick) => nick.into_inner(),
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
            Ok(ModeTarget::User(nickname))
        }
    }
}

impl fmt::Display for ModeTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModeTarget::Channel(channel) => write!(f, "{channel}"),
            ModeTarget::User(nick) => write!(f, "{nick}"),
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
                Ok(nickname) => Ok(ModeTarget::User(nickname)),
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
            ModeTarget::User(nick) => nick.as_ref(),
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
        ModeTarget::User(value)
    }
}

impl From<ModeTarget> for MedialParam {
    fn from(value: ModeTarget) -> MedialParam {
        MedialParam::try_from(value.into_inner()).expect("Mode target should be valid MedialParam")
    }
}

impl From<ModeTarget> for FinalParam {
    fn from(value: ModeTarget) -> FinalParam {
        FinalParam::from(MedialParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseModeTargetError {
    #[error(transparent)]
    Channel(#[from] ParseChannelError),
    #[error(transparent)]
    Nickname(#[from] ParseNicknameError),
}
