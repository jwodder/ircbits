use nutype::nutype;
use thiserror::Error;

#[nutype(
    validate(with = validate, error = VerbError),
    derive(AsRef, Clone, Debug, Deref, Display, Eq, FromStr, Into, PartialEq, TryFrom),
)]
pub struct Verb(String);

impl PartialEq<str> for Verb {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl<'a> PartialEq<&'a str> for Verb {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

fn validate(s: &str) -> Result<(), VerbError> {
    if s.is_empty() {
        Err(VerbError::Empty)
    } else if s.contains(|ch: char| !ch.is_ascii_alphabetic()) {
        Err(VerbError::BadCharacter)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum VerbError {
    #[error("verbs cannot be empty")]
    Empty,
    #[error("verbs may only contain letters")]
    BadCharacter,
}
