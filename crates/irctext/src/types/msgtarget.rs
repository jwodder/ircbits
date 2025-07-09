use crate::types::{
    channel::channel_prefixed, Channel, Nickname, ParseChannelError, ParseNicknameError,
};
use crate::TryFromStringError;
use thiserror::Error;

/// The target of a `PRIVMSG` or `NOTICE` message
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MsgTarget {
    Channel(Channel),
    Nick(Nickname),
    Star,
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
        match self {
            MsgTarget::Channel(chan) => chan.as_ref(),
            MsgTarget::Nick(nick) => nick.as_ref(),
            MsgTarget::Star => "*",
        }
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

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseMsgTargetError {
    #[error(transparent)]
    Channel(#[from] ParseChannelError),
    #[error(transparent)]
    Nickname(#[from] ParseNicknameError),
}
