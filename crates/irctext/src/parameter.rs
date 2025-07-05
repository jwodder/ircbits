use nutype::nutype;
use thiserror::Error;

#[nutype(
    validate(with = validate, error = ParameterError),
    derive(AsRef, Clone, Debug, Deref, Display, Eq, FromStr, Into, PartialEq, TryFrom),
)]
pub struct Parameter(String);

impl Parameter {
    pub fn is_middle(&self) -> bool {
        !self.is_final()
    }

    pub fn is_final(&self) -> bool {
        self.as_ref().is_empty() || self.as_ref().starts_with(':') || self.as_ref().contains(' ')
    }
}

impl PartialEq<str> for Parameter {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl<'a> PartialEq<&'a str> for Parameter {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

fn validate(s: &str) -> Result<(), ParameterError> {
    if s.contains(['\0', '\r', '\n']) {
        Err(ParameterError)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("parameters cannot contain NUL, CR, or LF")]
pub struct ParameterError;
