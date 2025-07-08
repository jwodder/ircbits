use crate::{FinalParam, MedialParam};
use thiserror::Error;

// Like a `MedialParam`, but not allowing commas
#[derive(Clone, Eq, PartialEq)]
pub struct EListCond(String);

validstr!(EListCond, ParseEListCondError, validate);

fn validate(s: &str) -> Result<(), ParseEListCondError> {
    if s.is_empty() {
        Err(ParseEListCondError::Empty)
    } else if s.starts_with(':') {
        Err(ParseEListCondError::StartsWithColon)
    } else if s.contains(['\0', '\r', '\n', ' ', ',']) {
        Err(ParseEListCondError::BadCharacter)
    } else {
        Ok(())
    }
}

impl From<EListCond> for MedialParam {
    fn from(value: EListCond) -> MedialParam {
        MedialParam::try_from(value.into_inner()).expect("EListCond should be valid MedialParam")
    }
}

impl From<EListCond> for FinalParam {
    fn from(value: EListCond) -> FinalParam {
        FinalParam::from(MedialParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseEListCondError {
    #[error("elistconds cannot be empty")]
    Empty,
    #[error("elistconds cannot start with a colon")]
    StartsWithColon,
    #[error("elistconds cannot contain NUL, CR, LF, SPACE, or COMMA")]
    BadCharacter,
}
