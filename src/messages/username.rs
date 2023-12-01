// See <https://github.com/ircdocs/modern-irc/issues/226> for notes on username
// format.
use std::borrow::Cow;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Username<'a>(Cow<'a, str>);

common_cow!(Username, UsernameError);

impl<'a> TryFrom<Cow<'a, str>> for Username<'a> {
    type Error = UsernameError;

    fn try_from(s: Cow<'a, str>) -> Result<Username<'a>, UsernameError> {
        if s.is_empty() {
            Err(UsernameError::Empty)
        } else if s.starts_with(':') {
            Err(UsernameError::StartsWithColon)
        } else if s.contains(['\0', '\r', '\n', ' ', '@']) {
            Err(UsernameError::BadCharacter)
        } else {
            Ok(Username(s))
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
