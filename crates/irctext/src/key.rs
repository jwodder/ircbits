// <https://modern.ircdocs.horse> does not specify a format for channel
// keys, leaving that large to individual server implementations, though we can
// deduce that they cannot contain commas.
use nutype::nutype;
use thiserror::Error;

#[nutype(
    validate(with = validate, error = KeyError),
    derive(AsRef, Clone, Debug, Deref, Display, Eq, FromStr, Into, PartialEq, TryFrom),
)]
pub struct Key(String);

impl PartialEq<str> for Key {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl<'a> PartialEq<&'a str> for Key {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

fn validate(s: &str) -> Result<(), KeyError> {
    if s.contains(['\0', '\r', '\n', ',']) {
        Err(KeyError)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("channel keys cannot contain NUL, CR, LF, or comma")]
pub struct KeyError;
