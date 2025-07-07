use crate::{Channel, Nickname, ParseChannelError, ParseNicknameError};
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
        // TODO: Improve this!
        } else if s.starts_with(['#', '&']) {
            let channel = s.parse::<Channel>()?;
            Ok(Target::Channel(channel))
        } else {
            let nickname = s.parse::<Nickname>()?;
            Ok(Target::User(nickname))
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
