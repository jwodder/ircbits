// <https://modern.ircdocs.horse> does not specify a format for channel
// keys, leaving that large to individual server implementations, though we can
// deduce that they cannot contain commas.
use thiserror::Error;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Key(String);

validstr!(Key, ParseKeyError, validate);

fn validate(s: &str) -> Result<(), ParseKeyError> {
    if s.contains(['\0', '\r', '\n', ',']) {
        Err(ParseKeyError)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("channel keys cannot contain NUL, CR, LF, or comma")]
pub struct ParseKeyError;
