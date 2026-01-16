use super::{MiddleParam, TrailingParam};
use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ParamRef<'a> {
    Middle(&'a MiddleParam),
    Trailing(&'a TrailingParam),
}

impl<'a> ParamRef<'a> {
    pub fn is_middle(&self) -> bool {
        matches!(self, ParamRef::Middle(_))
    }

    pub fn is_trailing(&self) -> bool {
        matches!(self, ParamRef::Trailing(_))
    }

    pub fn as_str(&self) -> &'a str {
        match self {
            ParamRef::Middle(p) => p.as_str(),
            ParamRef::Trailing(p) => p.as_str(),
        }
    }
}

impl fmt::Display for ParamRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamRef::Middle(p) => write!(f, "{p}"),
            ParamRef::Trailing(p) => write!(f, "{p}"),
        }
    }
}

impl AsRef<str> for ParamRef<'_> {
    fn as_ref(&self) -> &str {
        match self {
            ParamRef::Middle(p) => p.as_ref(),
            ParamRef::Trailing(p) => p.as_ref(),
        }
    }
}

impl PartialEq<String> for ParamRef<'_> {
    fn eq(&self, other: &String) -> bool {
        self.as_ref() == other.as_str()
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

impl From<ParamRef<'_>> for String {
    fn from(value: ParamRef<'_>) -> String {
        value.as_str().to_owned()
    }
}
