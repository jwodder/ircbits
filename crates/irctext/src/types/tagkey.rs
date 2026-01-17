// <https://ircv3.net/specs/extensions/message-tags.html> gives a grammar for
// tag keys, but it also says "Implementations … MUST NOT perform any
// validation that would reject the message if an invalid tag key name is used.
// This allows future modifications to the tag key name format."  So we're
// going to accept anything that could be used as a tag key name without
// affecting how the tags are parsed.
use thiserror::Error;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TagKey(String);

validstr!(TagKey, ParseTagKeyError, validate);
strserde!(TagKey, "an IRC tag key name");

fn validate(s: &str) -> Result<(), ParseTagKeyError> {
    if s.is_empty() {
        Err(ParseTagKeyError::Empty)
    } else if s.contains(['\0', '\r', '\n', ' ', ';', '=']) {
        Err(ParseTagKeyError::BadCharacter)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum ParseTagKeyError {
    #[error("tag key names cannot be empty")]
    Empty,
    #[error("tag key names cannot contain NUL, CR, LF, space, semicolon, or =")]
    BadCharacter,
}
