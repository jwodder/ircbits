use crate::{FinalParam, MedialParam};
use thiserror::Error;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModeString(String);

validstr!(ModeString, ParseModeStringError, validate);

fn validate(s: &str) -> Result<(), ParseModeStringError> {
    if !s.starts_with(['+', '-']) {
        Err(ParseModeStringError::BadStart)
    } else if s.contains(|c: char| !(c.is_ascii_alphabetic() || c == '+' || c == '-')) {
        Err(ParseModeStringError::BadCharacter)
    } else {
        Ok(())
    }
}

impl From<ModeString> for MedialParam {
    fn from(value: ModeString) -> MedialParam {
        MedialParam::try_from(value.into_inner()).expect("Mode string should be valid MedialParam")
    }
}

impl From<ModeString> for FinalParam {
    fn from(value: ModeString) -> FinalParam {
        FinalParam::from(MedialParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum ParseModeStringError {
    #[error("mode strings must start with + or -")]
    BadStart,
    #[error("mode strings can only contain +, -, and ASCII letters")]
    BadCharacter,
}
