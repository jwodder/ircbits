use std::borrow::Cow;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Parameter<'a>(Cow<'a, str>);

common_cow!(Parameter, ParameterError);

impl Parameter<'_> {
    pub(crate) fn is_middle(&self) -> bool {
        !self.is_final()
    }

    pub(crate) fn is_final(&self) -> bool {
        let s = self.0.as_ref();
        s.is_empty() || s.starts_with(':') || s.contains(' ')
    }
}

impl<'a> TryFrom<Cow<'a, str>> for Parameter<'a> {
    type Error = ParameterError;

    fn try_from(s: Cow<'a, str>) -> Result<Parameter<'a>, ParameterError> {
        if s.contains(['\0', '\r', '\n']) {
            Err(ParameterError)
        } else {
            Ok(Parameter(s))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("parameters cannot contain NUL, CR, or LF")]
pub(crate) struct ParameterError;
