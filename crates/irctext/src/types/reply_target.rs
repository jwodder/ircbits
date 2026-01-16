use crate::types::{Nickname, ParseNicknameError};
use crate::{MiddleParam, TrailingParam, TryFromStringError};
use std::fmt;
use thiserror::Error;

/// The target of a reply sent from a server to a client, including both
/// numeric replies and server-to-client `CAP` messages
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ReplyTarget {
    Nick(Nickname),
    Star,
}

impl ReplyTarget {
    pub fn is_nick(&self) -> bool {
        matches!(self, ReplyTarget::Nick(_))
    }

    pub fn is_star(&self) -> bool {
        matches!(self, ReplyTarget::Star)
    }

    pub fn as_str(&self) -> &str {
        match self {
            ReplyTarget::Nick(nick) => nick.as_str(),
            ReplyTarget::Star => "*",
        }
    }
}

impl fmt::Display for ReplyTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ReplyTarget {
    type Err = ParseReplyTargetError;

    fn from_str(s: &str) -> Result<ReplyTarget, ParseReplyTargetError> {
        if s == "*" {
            Ok(ReplyTarget::Star)
        } else {
            let nickname = s.parse::<Nickname>()?;
            Ok(ReplyTarget::Nick(nickname))
        }
    }
}

impl TryFrom<String> for ReplyTarget {
    type Error = TryFromStringError<ParseReplyTargetError>;

    fn try_from(value: String) -> Result<ReplyTarget, TryFromStringError<ParseReplyTargetError>> {
        if value == "*" {
            Ok(ReplyTarget::Star)
        } else {
            match Nickname::try_from(value) {
                Ok(nickname) => Ok(ReplyTarget::Nick(nickname)),
                Err(TryFromStringError { inner, string }) => Err(TryFromStringError {
                    inner: ParseReplyTargetError(inner),
                    string,
                }),
            }
        }
    }
}

impl AsRef<str> for ReplyTarget {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<String> for ReplyTarget {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<str> for ReplyTarget {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<'a> PartialEq<&'a str> for ReplyTarget {
    fn eq(&self, other: &&'a str) -> bool {
        self == *other
    }
}

impl PartialEq<Nickname> for ReplyTarget {
    fn eq(&self, other: &Nickname) -> bool {
        matches!(self, ReplyTarget::Nick(nick) if nick == other)
    }
}

impl From<Nickname> for ReplyTarget {
    fn from(value: Nickname) -> ReplyTarget {
        ReplyTarget::Nick(value)
    }
}

impl From<ReplyTarget> for String {
    fn from(value: ReplyTarget) -> String {
        match value {
            ReplyTarget::Nick(nick) => nick.into(),
            ReplyTarget::Star => String::from("*"),
        }
    }
}

impl From<ReplyTarget> for MiddleParam {
    fn from(value: ReplyTarget) -> MiddleParam {
        MiddleParam::try_from(String::from(value))
            .expect("Reply target should be valid MiddleParam")
    }
}

impl From<ReplyTarget> for TrailingParam {
    fn from(value: ReplyTarget) -> TrailingParam {
        TrailingParam::from(MiddleParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
#[error(transparent)]
pub struct ParseReplyTargetError(#[from] ParseNicknameError);
