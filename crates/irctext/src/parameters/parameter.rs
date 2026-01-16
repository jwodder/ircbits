use super::{MiddleParam, TrailingParam};
use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Parameter {
    Middle(MiddleParam),
    Trailing(TrailingParam),
}

impl Parameter {
    pub fn is_middle(&self) -> bool {
        matches!(self, Parameter::Middle(_))
    }

    pub fn is_trailing(&self) -> bool {
        matches!(self, Parameter::Trailing(_))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Parameter::Middle(p) => p.as_str(),
            Parameter::Trailing(p) => p.as_str(),
        }
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Parameter::Middle(p) => write!(f, "{p}"),
            Parameter::Trailing(p) => write!(f, "{p}"),
        }
    }
}

impl AsRef<str> for Parameter {
    fn as_ref(&self) -> &str {
        match self {
            Parameter::Middle(p) => p.as_ref(),
            Parameter::Trailing(p) => p.as_ref(),
        }
    }
}

impl From<Parameter> for String {
    fn from(value: Parameter) -> String {
        match value {
            Parameter::Middle(param) => param.into(),
            Parameter::Trailing(param) => param.into(),
        }
    }
}

impl PartialEq<String> for Parameter {
    fn eq(&self, other: &String) -> bool {
        self.as_ref() == other.as_str()
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

impl From<MiddleParam> for Parameter {
    fn from(value: MiddleParam) -> Parameter {
        Parameter::Middle(value)
    }
}

impl From<TrailingParam> for Parameter {
    fn from(value: TrailingParam) -> Parameter {
        Parameter::Trailing(value)
    }
}
