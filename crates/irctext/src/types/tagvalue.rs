use std::fmt;
use thiserror::Error;

#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TagValue(String);

// Display does not show escapes.  FromStr does not parse escapes.
validstr!(TagValue, ParseTagValueError, validate);
strserde!(TagValue, "an IRC tag value");

fn validate(s: &str) -> Result<(), ParseTagValueError> {
    if s.contains(['\0', '\r', '\n', ' ', ';']) {
        Err(ParseTagValueError::BadCharacter)
    } else {
        Ok(())
    }
}

impl TagValue {
    pub fn from_escaped(s: &str) -> Result<TagValue, ParseTagValueError> {
        let mut value = String::with_capacity(s.len());
        let mut chars = s.chars();
        let iter = chars.by_ref();
        while let Some(ch) = iter.next() {
            match ch {
                '\\' => match iter.next() {
                    Some(':') => value.push(';'),
                    Some('s') => value.push(' '),
                    Some('\\') => value.push('\\'),
                    Some('r') => value.push('\r'),
                    Some('n') => value.push('\n'),
                    Some('\0' | '\r' | '\n' | ' ') => return Err(ParseTagValueError::BadCharacter),
                    Some(c) => value.push(c),
                    None => (),
                },
                '\0' | '\r' | '\n' | ' ' | ';' => return Err(ParseTagValueError::BadCharacter),
                c => value.push(c),
            }
        }
        Ok(TagValue(value))
    }

    pub fn escaped(&self) -> EscapedTagValue<'_> {
        EscapedTagValue(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum ParseTagValueError {
    #[error("escaped tag values cannot contain NUL, CR, LF, space, or semicolon")]
    BadCharacter,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EscapedTagValue<'a>(&'a TagValue);

impl fmt::Display for EscapedTagValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0.0.as_str();
        let mut prev_start = 0;
        for (i, m) in s.match_indices([';', ' ', '\\', '\r', '\n']) {
            write!(f, "{}", &s[prev_start..i])?;
            match m {
                ";" => write!(f, "\\:")?,
                " " => write!(f, "\\s")?,
                "\\" => write!(f, "\\\\")?,
                "\r" => write!(f, "\\r")?,
                "\n" => write!(f, "\\n")?,
                _ => unreachable!("Only SEMICOLON, SPACE, BACKSLASH, CR, and LF should be matched"),
            }
            prev_start = i + 1;
        }
        write!(f, "{}", &s[prev_start..])?;
        Ok(())
    }
}
