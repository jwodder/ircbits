use super::{FinalParam, MedialParam};
use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Parameter {
    Medial(MedialParam),
    Final(FinalParam),
}

impl Parameter {
    pub fn is_medial(&self) -> bool {
        matches!(self, Parameter::Medial(_))
    }

    pub fn is_final(&self) -> bool {
        matches!(self, Parameter::Final(_))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Parameter::Medial(p) => p.as_str(),
            Parameter::Final(p) => p.as_str(),
        }
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Parameter::Medial(p) => write!(f, "{p}"),
            Parameter::Final(p) => write!(f, "{p}"),
        }
    }
}

impl AsRef<str> for Parameter {
    fn as_ref(&self) -> &str {
        match self {
            Parameter::Medial(p) => p.as_ref(),
            Parameter::Final(p) => p.as_ref(),
        }
    }
}

impl From<Parameter> for String {
    fn from(value: Parameter) -> String {
        match value {
            Parameter::Medial(param) => param.into(),
            Parameter::Final(param) => param.into(),
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

impl From<MedialParam> for Parameter {
    fn from(value: MedialParam) -> Parameter {
        Parameter::Medial(value)
    }
}

impl From<FinalParam> for Parameter {
    fn from(value: FinalParam) -> Parameter {
        Parameter::Final(value)
    }
}
