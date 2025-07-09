use crate::types::{Nickname, ParseNicknameError};
use crate::TryFromStringError;
use std::fmt;
use thiserror::Error;

/// The target of a reply sent from a server to a client, including both
/// numeric replies and server-to-client `CAP` messages
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplyTarget {
    User(Nickname),
    Star,
}

impl ReplyTarget {
    pub fn into_inner(self) -> String {
        match self {
            ReplyTarget::User(nick) => nick.into_inner(),
            ReplyTarget::Star => String::from("*"),
        }
    }
}

impl std::str::FromStr for ReplyTarget {
    type Err = ParseReplyTargetError;

    fn from_str(s: &str) -> Result<ReplyTarget, ParseReplyTargetError> {
        if s == "*" {
            Ok(ReplyTarget::Star)
        } else {
            let nickname = s.parse::<Nickname>()?;
            Ok(ReplyTarget::User(nickname))
        }
    }
}

impl fmt::Display for ReplyTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReplyTarget::User(nick) => write!(f, "{nick}"),
            ReplyTarget::Star => write!(f, "*"),
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
                Ok(nickname) => Ok(ReplyTarget::User(nickname)),
                Err(TryFromStringError { inner, string }) => Err(TryFromStringError {
                    inner: ParseReplyTargetError(inner),
                    string,
                }),
            }
        }
    }
}

impl PartialEq<str> for ReplyTarget {
    fn eq(&self, other: &str) -> bool {
        match self {
            ReplyTarget::User(nick) => nick == other,
            ReplyTarget::Star => other == "*",
        }
    }
}

impl<'a> PartialEq<&'a str> for ReplyTarget {
    fn eq(&self, other: &&'a str) -> bool {
        self == *other
    }
}

impl AsRef<str> for ReplyTarget {
    fn as_ref(&self) -> &str {
        match self {
            ReplyTarget::User(nick) => nick.as_ref(),
            ReplyTarget::Star => "*",
        }
    }
}

impl From<Nickname> for ReplyTarget {
    fn from(value: Nickname) -> ReplyTarget {
        ReplyTarget::User(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error(transparent)]
pub struct ParseReplyTargetError(#[from] ParseNicknameError);
