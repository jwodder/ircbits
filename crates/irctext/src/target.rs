use crate::channel::channel_prefixed;
use crate::{Channel, Nickname, ParseChannelError, ParseNicknameError, TryFromStringError};
use thiserror::Error;

/// The target of a `PRIVMSG` or `NOTICE` message
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Target {
    Channel(Channel),
    User(Nickname),
    Star,
}

impl std::str::FromStr for Target {
    type Err = ParseTargetError;

    fn from_str(s: &str) -> Result<Target, ParseTargetError> {
        if s == "*" {
            Ok(Target::Star)
        } else if channel_prefixed(s) {
            let channel = s.parse::<Channel>()?;
            Ok(Target::Channel(channel))
        } else {
            let nickname = s.parse::<Nickname>()?;
            Ok(Target::User(nickname))
        }
    }
}

impl TryFrom<String> for Target {
    type Error = TryFromStringError<ParseTargetError>;

    fn try_from(value: String) -> Result<Target, TryFromStringError<ParseTargetError>> {
        if value == "*" {
            Ok(Target::Star)
        } else if channel_prefixed(&value) {
            match Channel::try_from(value) {
                Ok(channel) => Ok(Target::Channel(channel)),
                Err(TryFromStringError { inner, string }) => Err(TryFromStringError {
                    inner: ParseTargetError::Channel(inner),
                    string,
                }),
            }
        } else {
            match Nickname::try_from(value) {
                Ok(nickname) => Ok(Target::User(nickname)),
                Err(TryFromStringError { inner, string }) => Err(TryFromStringError {
                    inner: ParseTargetError::Nickname(inner),
                    string,
                }),
            }
        }
    }
}

impl AsRef<str> for Target {
    fn as_ref(&self) -> &str {
        match self {
            Target::Channel(chan) => chan.as_ref(),
            Target::User(nick) => nick.as_ref(),
            Target::Star => "*",
        }
    }
}

impl From<Channel> for Target {
    fn from(value: Channel) -> Target {
        Target::Channel(value)
    }
}

impl From<Nickname> for Target {
    fn from(value: Nickname) -> Target {
        Target::User(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseTargetError {
    #[error(transparent)]
    Channel(#[from] ParseChannelError),
    #[error(transparent)]
    Nickname(#[from] ParseNicknameError),
}
