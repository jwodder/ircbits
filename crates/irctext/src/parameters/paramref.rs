use super::{FinalParam, MedialParam};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParamRef<'a> {
    Medial(&'a MedialParam),
    Final(&'a FinalParam),
}

impl ParamRef<'_> {
    pub fn is_medial(&self) -> bool {
        matches!(self, ParamRef::Medial(_))
    }

    pub fn is_final(&self) -> bool {
        matches!(self, ParamRef::Final(_))
    }

    pub fn as_str(&self) -> &str {
        match self {
            ParamRef::Medial(p) => p.as_str(),
            ParamRef::Final(p) => p.as_str(),
        }
    }
}

impl fmt::Display for ParamRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamRef::Medial(p) => write!(f, "{p}"),
            ParamRef::Final(p) => write!(f, "{p}"),
        }
    }
}

impl AsRef<str> for ParamRef<'_> {
    fn as_ref(&self) -> &str {
        match self {
            ParamRef::Medial(p) => p.as_ref(),
            ParamRef::Final(p) => p.as_ref(),
        }
    }
}

impl PartialEq<str> for ParamRef<'_> {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl<'a> PartialEq<&'a str> for ParamRef<'_> {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}
