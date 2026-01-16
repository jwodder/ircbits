// See <https://github.com/ircdocs/modern-irc/issues/226> for notes on username
// format.
use crate::{MiddleParam, TrailingParam};
use thiserror::Error;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Username(String);

validstr!(Username, ParseUsernameError, validate);
strserde!(Username, "an IRC username");

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

impl From<Username> for MiddleParam {
    fn from(value: Username) -> MiddleParam {
        MiddleParam::try_from(value.into_inner()).expect("Username should be valid MiddleParam")
    }
}

impl From<Username> for TrailingParam {
    fn from(value: Username) -> TrailingParam {
        TrailingParam::from(MiddleParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum ParseUsernameError {
    #[error("usernames cannot be empty")]
    Empty,
    #[error("usernames cannot start with a colon")]
    StartsWithColon,
    #[error("usernames cannot contain NUL, CR, LF, SPACE, or @")]
    BadCharacter,
}
