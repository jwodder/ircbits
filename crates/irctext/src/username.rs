// See <https://github.com/ircdocs/modern-irc/issues/226> for notes on username
// format.
use crate::parameters::{FinalParam, MedialParam};
use nutype::nutype;
use thiserror::Error;

#[nutype(
    validate(with = validate, error = UsernameError),
    derive(AsRef, Clone, Debug, Deref, Display, Eq, FromStr, Into, PartialEq, TryFrom),
)]
pub struct Username(String);

impl PartialEq<str> for Username {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl<'a> PartialEq<&'a str> for Username {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

fn validate(s: &str) -> Result<(), UsernameError> {
    if s.is_empty() {
        Err(UsernameError::Empty)
    } else if s.starts_with(':') {
        Err(UsernameError::StartsWithColon)
    } else if s.contains(['\0', '\r', '\n', ' ', '@']) {
        Err(UsernameError::BadCharacter)
    } else {
        Ok(())
    }
}

impl From<Username> for MedialParam {
    fn from(value: Username) -> MedialParam {
        MedialParam::try_from(value.into_inner()).expect("Username should be valid MedialParam")
    }
}

impl From<Username> for FinalParam {
    fn from(value: Username) -> FinalParam {
        FinalParam::from(MedialParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum UsernameError {
    #[error("usernames cannot be empty")]
    Empty,
    #[error("usernames cannot start with a colon")]
    StartsWithColon,
    #[error("usernames cannot contain NUL, CR, LF, SPACE, or @")]
    BadCharacter,
}
