// See <https://github.com/ircdocs/modern-irc/issues/226> for notes on username
// format.
use std::borrow::Cow;
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Username<'a>(Cow<'a, str>);

impl fmt::Display for Username<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<str> for Username<'_> {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl<'a> PartialEq<&'a str> for Username<'_> {
    fn eq(&self, other: &&'a str) -> bool {
        &self.0 == other
    }
}

impl AsRef<str> for Username<'_> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'a> TryFrom<&'a str> for Username<'a> {
    type Error = UsernameError;

    fn try_from(s: &'a str) -> Result<Username<'a>, UsernameError> {
        if s.is_empty() {
            Err(UsernameError::Empty)
        } else if s.starts_with(':') {
            Err(UsernameError::StartsWithColon)
        } else if s.contains(['\0', '\r', '\n', ' ', '@']) {
            Err(UsernameError::BadCharacter)
        } else {
            Ok(Username(s.into()))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum UsernameError {
    #[error("usernames cannot be empty")]
    Empty,
    #[error("usernames cannot start with a colon")]
    StartsWithColon,
    #[error("usernames cannot contain NUL, CR, LF, SPACE, or @")]
    BadCharacter,
}
