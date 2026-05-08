use crate::{MiddleParam, TrailingParam};
use thiserror::Error;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Capability(String);

validstr!(Capability, ParseCapabilityError, validate_capability);
strserde!(Capability, "an IRCv3 capability name");

fn validate_capability(s: &str) -> Result<(), ParseCapabilityError> {
    if s.is_empty() {
        Err(ParseCapabilityError::Empty)
    } else if s.starts_with('-') {
        Err(ParseCapabilityError::BadStart)
    } else if s.contains(['\0', '\r', '\n', ' ', '=']) {
        Err(ParseCapabilityError::BadCharacter)
    } else {
        Ok(())
    }
}

impl From<Capability> for MiddleParam {
    fn from(value: Capability) -> MiddleParam {
        MiddleParam::try_from(value.into_inner()).expect("Capability should be valid MiddleParam")
    }
}

impl From<Capability> for TrailingParam {
    fn from(value: Capability) -> TrailingParam {
        TrailingParam::from(MiddleParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseCapabilityError {
    #[error("capabilities cannot be empty")]
    Empty,
    #[error("capabilities cannot start with '-'")]
    BadStart,
    #[error("capabilities cannot contain NUL, CR, LF, SPACE, or =")]
    BadCharacter,
}
