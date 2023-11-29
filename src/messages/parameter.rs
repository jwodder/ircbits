use std::borrow::Cow;
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Parameter<'a>(Cow<'a, str>);

impl Parameter<'_> {
    pub(crate) fn is_middle(&self) -> bool {
        !self.is_final()
    }

    pub(crate) fn is_final(&self) -> bool {
        let s = self.0.as_ref();
        s.is_empty() || s.starts_with(':') || s.contains(' ')
    }
}

impl fmt::Display for Parameter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<str> for Parameter<'_> {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl<'a> PartialEq<&'a str> for Parameter<'_> {
    fn eq(&self, other: &&'a str) -> bool {
        &self.0 == other
    }
}

impl AsRef<str> for Parameter<'_> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'a> TryFrom<&'a str> for Parameter<'a> {
    type Error = ParameterError;

    fn try_from(s: &'a str) -> Result<Parameter<'a>, ParameterError> {
        if s.contains(['\0', '\r', '\n']) {
            Err(ParameterError)
        } else {
            Ok(Parameter(s.into()))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("parameters cannot contain NUL, CR, or LF")]
pub(crate) struct ParameterError;
