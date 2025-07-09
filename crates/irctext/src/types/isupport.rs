use crate::{FinalParam, MedialParam, TryFromStringError};
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ISupportParam {
    Set(ISupportKey),
    Unset(ISupportKey),
    Eq(ISupportKey, ISupportValue),
}

impl ISupportParam {
    pub fn is_set(&self) -> bool {
        matches!(self, ISupportParam::Set(_))
    }

    pub fn is_unset(&self) -> bool {
        matches!(self, ISupportParam::Unset(_))
    }

    pub fn is_eq(&self) -> bool {
        matches!(self, ISupportParam::Eq(_, _))
    }

    pub fn key(&self) -> &ISupportKey {
        match self {
            ISupportParam::Set(key) => key,
            ISupportParam::Unset(key) => key,
            ISupportParam::Eq(key, _) => key,
        }
    }

    pub fn value(&self) -> Option<&ISupportValue> {
        match self {
            ISupportParam::Eq(_, value) => Some(value),
            _ => None,
        }
    }
}

impl fmt::Display for ISupportParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ISupportParam::Set(key) => write!(f, "{key}"),
            ISupportParam::Unset(key) => write!(f, "-{key}"),
            ISupportParam::Eq(key, value) => write!(f, "{key}={}", value.escaped()),
        }
    }
}

impl std::str::FromStr for ISupportParam {
    type Err = ParseISupportParamError;

    fn from_str(s: &str) -> Result<ISupportParam, ParseISupportParamError> {
        if let Some((key, value)) = s.split_once('=') {
            let key = key.parse::<ISupportKey>()?;
            let value = value.parse::<ISupportValue>()?;
            Ok(ISupportParam::Eq(key, value))
        } else if let Some(key) = s.strip_prefix('-') {
            let key = key.parse::<ISupportKey>()?;
            Ok(ISupportParam::Unset(key))
        } else {
            let key = s.parse::<ISupportKey>()?;
            Ok(ISupportParam::Set(key))
        }
    }
}

impl TryFrom<String> for ISupportParam {
    type Error = TryFromStringError<ParseISupportParamError>;

    fn try_from(
        string: String,
    ) -> Result<ISupportParam, TryFromStringError<ParseISupportParamError>> {
        match string.parse() {
            Ok(src) => Ok(src),
            Err(inner) => Err(TryFromStringError { inner, string }),
        }
    }
}

impl From<ISupportParam> for MedialParam {
    fn from(value: ISupportParam) -> MedialParam {
        MedialParam::try_from(value.to_string()).expect("ISupportParam should be valid MedialParam")
    }
}

impl From<ISupportParam> for FinalParam {
    fn from(value: ISupportParam) -> FinalParam {
        FinalParam::from(MedialParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseISupportParamError {
    #[error("invalid ISUPPORT key")]
    Key(#[from] ParseISupportKeyError),

    #[error("invalid ISUPPORT value")]
    Value(#[from] ParseISupportValueError),
}

// modern.ircdocs.horse says that ISUPPORT keys should be limited to 20
// characters, but I'm not going to enforce that.
#[derive(Clone, Eq, PartialEq)]
pub struct ISupportKey(String);

validstr!(ISupportKey, ParseISupportKeyError, validate);

fn validate(s: &str) -> Result<(), ParseISupportKeyError> {
    if s.is_empty() {
        Err(ParseISupportKeyError::Empty)
    } else if s.contains(|ch: char| !ch.is_ascii_alphanumeric()) {
        Err(ParseISupportKeyError::BadCharacter)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseISupportKeyError {
    #[error("ISUPPORT keys cannot be empty")]
    Empty,
    #[error("ISUPPORT keys must only contain ASCII letters & digits")]
    BadCharacter,
}

#[derive(Clone, Eq, PartialEq)]
pub struct ISupportValue(String);

impl ISupportValue {
    pub fn escaped(&self) -> EscapedISupportValue<'_> {
        EscapedISupportValue(self)
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<ISupportValue> for String {
    fn from(value: ISupportValue) -> String {
        value.0
    }
}

impl From<&ISupportValue> for String {
    fn from(value: &ISupportValue) -> String {
        value.0.clone()
    }
}

impl fmt::Debug for ISupportValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

// Displays WITHOUT escapes
impl fmt::Display for ISupportValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Parses escapes
impl std::str::FromStr for ISupportValue {
    type Err = ParseISupportValueError;

    fn from_str(s: &str) -> Result<ISupportValue, ParseISupportValueError> {
        let mut value = String::with_capacity(s.len());
        let mut chars = s.chars();
        let iter = chars.by_ref();
        while let Some(ch) = iter.next() {
            if !ch.is_ascii_graphic() {
                return Err(ParseISupportValueError::BadCharacter);
            }
            if ch == '\\' {
                let Some('x') = iter.next() else {
                    return Err(ParseISupportValueError::BadEscape);
                };
                match (iter.next(), iter.next()) {
                    (Some('2'), Some('0')) => value.push(' '),
                    (Some('3'), Some('D' | 'd')) => value.push('='),
                    (Some('5'), Some('C' | 'c')) => value.push('\\'),
                    _ => return Err(ParseISupportValueError::BadEscape),
                }
            } else {
                value.push(ch);
            }
        }
        Ok(ISupportValue(value))
    }
}

impl TryFrom<String> for ISupportValue {
    type Error = TryFromStringError<ParseISupportValueError>;

    fn try_from(
        string: String,
    ) -> Result<ISupportValue, TryFromStringError<ParseISupportValueError>> {
        match string.parse() {
            Ok(src) => Ok(src),
            Err(inner) => Err(TryFromStringError { inner, string }),
        }
    }
}

impl AsRef<str> for ISupportValue {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::ops::Deref for ISupportValue {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<String> for ISupportValue {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

impl PartialEq<str> for ISupportValue {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl<'a> PartialEq<&'a str> for ISupportValue {
    fn eq(&self, other: &&'a str) -> bool {
        &self.0 == other
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseISupportValueError {
    #[error("ISUPPORT values must only contain printable non-space ASCII characters")]
    BadCharacter,
    #[error("invalid/unrecognized escape sequence")]
    BadEscape,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EscapedISupportValue<'a>(&'a ISupportValue);

impl fmt::Display for EscapedISupportValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0 .0.as_str();
        let mut prev_start = 0;
        for (i, m) in s.match_indices([' ', '=', '\\']) {
            write!(f, "{}", &s[prev_start..i])?;
            match m {
                " " => write!(f, "\\x20")?,
                "=" => write!(f, "\\x3D")?,
                "\\" => write!(f, "\\x5C")?,
                _ => unreachable!("Only SPACE, BACKSLASH, and EQ should be matched"),
            }
            prev_start = i + 1;
        }
        write!(f, "{}", &s[prev_start..])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn parse_set() {
        let isp = "EXCEPTS".parse::<ISupportParam>().unwrap();
        assert_matches!(isp, ISupportParam::Set(key) => {
            assert_eq!(key, "EXCEPTS");
        });
    }

    #[test]
    fn parse_unset() {
        let isp = "-EXCEPTS".parse::<ISupportParam>().unwrap();
        assert_matches!(isp, ISupportParam::Unset(key) => {
            assert_eq!(key, "EXCEPTS");
        });
    }

    #[test]
    fn parse_eq() {
        let isp = "CHANTYPES=#".parse::<ISupportParam>().unwrap();
        assert_matches!(isp, ISupportParam::Eq(key, value) => {
            assert_eq!(key, "CHANTYPES");
            assert_eq!(value, "#");
        });
    }

    #[test]
    fn escaped_value() {
        let s = r"foo\x3Dbar\x5Cbaz\x20quux";
        let value = s.parse::<ISupportValue>().unwrap();
        assert_eq!(value.as_ref(), "foo=bar\\baz quux");
        assert_eq!(value.to_string(), "foo=bar\\baz quux");
        assert_eq!(value.escaped().to_string(), s);
    }

    #[test]
    fn lower_escaped_value() {
        let s = r"foo\x3dbar\x5cbaz\x20quux";
        let value = s.parse::<ISupportValue>().unwrap();
        assert_eq!(value.as_ref(), "foo=bar\\baz quux");
        assert_eq!(value.to_string(), "foo=bar\\baz quux");
        assert_eq!(value.escaped().to_string(), r"foo\x3Dbar\x5Cbaz\x20quux");
    }
}
