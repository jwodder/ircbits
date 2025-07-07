use crate::parameters::{FinalParam, MedialParam};
use thiserror::Error;

#[derive(Clone, Eq, PartialEq)]
pub struct Modestring(String);

validstr!(Modestring, ParseModestringError, validate);

fn validate(s: &str) -> Result<(), ParseModestringError> {
    if !s.starts_with(['+', '-']) {
        Err(ParseModestringError::BadStart)
    } else if s.contains(|c: char| !(c.is_ascii_alphabetic() || c == '+' || c == '-')) {
        Err(ParseModestringError::BadCharacter)
    } else {
        Ok(())
    }
}

impl From<Modestring> for MedialParam {
    fn from(value: Modestring) -> MedialParam {
        MedialParam::try_from(value.into_inner()).expect("Modestring should be valid MedialParam")
    }
}

impl From<Modestring> for FinalParam {
    fn from(value: Modestring) -> FinalParam {
        FinalParam::from(MedialParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseModestringError {
    #[error("modestrings must start with + or -")]
    BadStart,
    #[error("modestrings can only contain +, -, and ASCII letters")]
    BadCharacter,
}
