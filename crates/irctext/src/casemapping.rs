use crate::TryFromStringError;
use std::borrow::Cow;
use std::fmt;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CaseMapping {
    Ascii,
    #[default]
    Rfc1459,
    Rfc1459Strict,
}

impl CaseMapping {
    pub fn lowercase_char(self, ch: char) -> char {
        match (self, ch) {
            (_, ch) if ch.is_ascii_alphabetic() => ch.to_ascii_lowercase(),
            (CaseMapping::Rfc1459 | CaseMapping::Rfc1459Strict, '[') => '{',
            (CaseMapping::Rfc1459 | CaseMapping::Rfc1459Strict, ']') => '}',
            (CaseMapping::Rfc1459 | CaseMapping::Rfc1459Strict, '\\') => '|',
            (CaseMapping::Rfc1459, '~') => '^',
            (_, ch) => ch,
        }
    }

    pub fn lowercase_str<'a>(self, s: &'a str) -> Cow<'a, str> {
        if let Some(i) = s
            .char_indices()
            .find_map(|(i, ch)| (self.lowercase_char(ch) != ch).then_some(i))
        {
            let mut s2 = s[..i].to_owned();
            for ch in s[i..].chars() {
                s2.push(self.lowercase_char(ch));
            }
            Cow::from(s2)
        } else {
            Cow::from(s)
        }
    }

    pub fn eq_ignore_case(self, s1: &str, s2: &str) -> bool {
        s1.len() == s2.len()
            && std::iter::zip(s1.chars(), s2.chars())
                .all(|(c1, c2)| self.lowercase_char(c1) == self.lowercase_char(c2))
    }
}

impl fmt::Display for CaseMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            CaseMapping::Ascii => "ascii",
            CaseMapping::Rfc1459 => "rfc1459",
            CaseMapping::Rfc1459Strict => "rfc1459-strict",
        };
        f.pad(name)
    }
}

impl std::str::FromStr for CaseMapping {
    type Err = ParseCaseMappingError;

    fn from_str(s: &str) -> Result<CaseMapping, ParseCaseMappingError> {
        match s {
            "ascii" => Ok(CaseMapping::Ascii),
            "rfc1459" => Ok(CaseMapping::Rfc1459),
            "rfc1459-strict" => Ok(CaseMapping::Rfc1459Strict),
            _ => Err(ParseCaseMappingError),
        }
    }
}

impl TryFrom<String> for CaseMapping {
    type Error = TryFromStringError<ParseCaseMappingError>;

    fn try_from(string: String) -> Result<CaseMapping, TryFromStringError<ParseCaseMappingError>> {
        match string.parse() {
            Ok(src) => Ok(src),
            Err(inner) => Err(TryFromStringError { inner, string }),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
#[error("unknown/unrecognized CASEMAPPING name")]
pub struct ParseCaseMappingError;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("", "", false)]
    #[case("foo", "foo", false)]
    #[case("FOO", "foo", true)]
    #[case("foo{bar}", "foo{bar}", false)]
    #[case("foo[bar]", "foo{bar}", true)]
    fn rfc1459_lowercase_str(#[case] before: &str, #[case] after: &str, #[case] changed: bool) {
        let r = CaseMapping::Rfc1459.lowercase_str(before);
        assert_eq!(r, after);
        assert_eq!(matches!(r, Cow::Owned(_)), changed);
    }
}
