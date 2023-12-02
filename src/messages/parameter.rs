use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Parameter(String);

common_string!(Parameter, ParameterError);

impl Parameter {
    pub(crate) fn is_middle(&self) -> bool {
        !self.is_final()
    }

    pub(crate) fn is_final(&self) -> bool {
        self.0.is_empty() || self.0.starts_with(':') || self.0.contains(' ')
    }
}

impl TryFrom<String> for Parameter {
    type Error = ParameterError;

    fn try_from(s: String) -> Result<Parameter, ParameterError> {
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
