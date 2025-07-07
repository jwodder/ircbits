use crate::{Nickname, ParseNicknameError, TryFromStringError};
use std::fmt;
use thiserror::Error;

/// The target of a server-to-client `CAP` message
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CapTarget {
    User(Nickname),
    Star,
}

impl CapTarget {
    pub fn into_inner(self) -> String {
        match self {
            CapTarget::User(nick) => nick.into_inner(),
            CapTarget::Star => String::from("*"),
        }
    }
}

impl std::str::FromStr for CapTarget {
    type Err = ParseCapTargetError;

    fn from_str(s: &str) -> Result<CapTarget, ParseCapTargetError> {
        if s == "*" {
            Ok(CapTarget::Star)
        } else {
            let nickname = s.parse::<Nickname>()?;
            Ok(CapTarget::User(nickname))
        }
    }
}

impl fmt::Display for CapTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CapTarget::User(nick) => write!(f, "{nick}"),
            CapTarget::Star => write!(f, "*"),
        }
    }
}

impl TryFrom<String> for CapTarget {
    type Error = TryFromStringError<ParseCapTargetError>;

    fn try_from(value: String) -> Result<CapTarget, TryFromStringError<ParseCapTargetError>> {
        if value == "*" {
            Ok(CapTarget::Star)
        } else {
            match Nickname::try_from(value) {
                Ok(nickname) => Ok(CapTarget::User(nickname)),
                Err(TryFromStringError { inner, string }) => Err(TryFromStringError {
                    inner: ParseCapTargetError(inner),
                    string,
                }),
            }
        }
    }
}

impl AsRef<str> for CapTarget {
    fn as_ref(&self) -> &str {
        match self {
            CapTarget::User(nick) => nick.as_ref(),
            CapTarget::Star => "*",
        }
    }
}

impl From<Nickname> for CapTarget {
    fn from(value: Nickname) -> CapTarget {
        CapTarget::User(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error(transparent)]
pub struct ParseCapTargetError(#[from] ParseNicknameError);
