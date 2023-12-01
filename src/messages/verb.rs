use std::borrow::Cow;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Verb<'a>(Cow<'a, str>);

common_cow!(Verb, VerbError);

impl<'a> TryFrom<Cow<'a, str>> for Verb<'a> {
    type Error = VerbError;

    fn try_from(s: Cow<'a, str>) -> Result<Verb<'a>, VerbError> {
        if s.is_empty() {
            Err(VerbError::Empty)
        } else if s.contains(|ch: char| !ch.is_ascii_alphabetic()) {
            Err(VerbError::BadCharacter)
        } else {
            Ok(Verb(s))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum VerbError {
    #[error("command names cannot be empty")]
    Empty,
    #[error("command names may only contain letters")]
    BadCharacter,
}
