// See <https://github.com/ircdocs/modern-irc/issues/226> for notes on username
// format.
use crate::{FinalParam, MedialParam};
use thiserror::Error;

#[derive(Clone, Eq, PartialEq)]
pub struct Username(String);

validstr!(Username, ParseUsernameError, validate);

fn validate(s: &str) -> Result<(), ParseUsernameError> {
    if s.is_empty() {
        Err(ParseUsernameError::Empty)
    } else if s.starts_with(':') {
        Err(ParseUsernameError::StartsWithColon)
    } else if s.contains(['\0', '\r', '\n', ' ', '@']) {
        Err(ParseUsernameError::BadCharacter)
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
pub enum ParseUsernameError {
    #[error("usernames cannot be empty")]
    Empty,
    #[error("usernames cannot start with a colon")]
    StartsWithColon,
    #[error("usernames cannot contain NUL, CR, LF, SPACE, or @")]
    BadCharacter,
}
