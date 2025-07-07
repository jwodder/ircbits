use nutype::nutype;
use thiserror::Error;

#[nutype(
    validate(with = validate, error = ParseMedialParamError),
    derive(AsRef, Clone, Debug, Deref, Display, Eq, FromStr, Into, PartialEq, TryFrom),
)]
pub struct MedialParam(String);

impl PartialEq<str> for MedialParam {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl<'a> PartialEq<&'a str> for MedialParam {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

fn validate(s: &str) -> Result<(), ParseMedialParamError> {
    if s.is_empty() {
        Err(ParseMedialParamError::Empty)
    } else if s.starts_with(':') {
        Err(ParseMedialParamError::StartsWithColon)
    } else if s.contains(['\0', '\r', '\n', ' ']) {
        Err(ParseMedialParamError::BadCharacter)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseMedialParamError {
    #[error("medial parameters cannot be empty")]
    Empty,
    #[error("medial parameters cannot start with a colon")]
    StartsWithColon,
    #[error("medial parameters cannot contain NUL, CR, LF, or SPACE")]
    BadCharacter,
}
