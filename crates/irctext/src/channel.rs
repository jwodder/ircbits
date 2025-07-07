// <https://modern.ircdocs.horse/#channels>:
//
// > Apart from the requirement of the first character being a valid channel
// > type prefix character; the only restriction on a channel name is that it
// > may not contain any spaces `(' ', 0x20)`, a control G / `BELL` `('^G',
// > 0x07)`, or a comma `(',', 0x2C)` (which is used as a list item separator
// > by the protocol).
//
// Note that this implementation does not enforce the requirement on the first
// character, as the set of valid channel type prefixes varies from server to
// server, and we have chosen to implement the `JOIN 0` command by specifying a
// prefixless channel of "0" for the "JOIN" verb.  However, this implementation
// does require that channel names not start with a colon (':', 0x3A), which is
// necessary in order to be able to pass parameters after a channel parameter.
use crate::{FinalParam, MedialParam};
use nutype::nutype;
use thiserror::Error;

#[nutype(
    validate(with = validate, error = ParseChannelError),
    derive(AsRef, Clone, Debug, Deref, Display, Eq, FromStr, Into, PartialEq, TryFrom),
)]
pub struct Channel(String);

impl PartialEq<str> for Channel {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl<'a> PartialEq<&'a str> for Channel {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

fn validate(s: &str) -> Result<(), ParseChannelError> {
    if s.is_empty() {
        Err(ParseChannelError::Empty)
    } else if s.starts_with(':') {
        Err(ParseChannelError::StartsWithColon)
    } else if s.contains(['\0', '\r', '\n', ' ', '\x07', ',']) {
        Err(ParseChannelError::BadCharacter)
    } else {
        Ok(())
    }
}

impl From<Channel> for MedialParam {
    fn from(value: Channel) -> MedialParam {
        MedialParam::try_from(value.into_inner()).expect("Channel should be valid MedialParam")
    }
}

impl From<Channel> for FinalParam {
    fn from(value: Channel) -> FinalParam {
        FinalParam::from(MedialParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseChannelError {
    #[error("channels cannot be empty")]
    Empty,
    #[error("channels cannot start with a colon")]
    StartsWithColon,
    #[error("channels cannot contain NUL, CR, LF, SPACE, BELL, or comma")]
    BadCharacter,
}
