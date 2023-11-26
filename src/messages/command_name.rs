use std::borrow::Cow;
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommandName<'a>(Cow<'a, str>);

impl fmt::Display for CommandName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<str> for CommandName<'_> {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl<'a> PartialEq<&'a str> for CommandName<'_> {
    fn eq(&self, other: &&'a str) -> bool {
        &self.0 == other
    }
}

impl AsRef<str> for CommandName<'_> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'a> TryFrom<&'a str> for CommandName<'a> {
    type Error = CommandNameError;

    fn try_from(s: &'a str) -> Result<CommandName<'a>, CommandNameError> {
        if s.is_empty() {
            Err(CommandNameError::Empty)
        } else if s.contains(|ch: char| !ch.is_ascii_alphabetic()) {
            Err(CommandNameError::BadCharacter)
        } else {
            Ok(CommandName(s.into()))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum CommandNameError {
    #[error("command names cannot be empty")]
    Empty,
    #[error("command names may only contain letters")]
    BadCharacter,
}
