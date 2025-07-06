// <https://modern.ircdocs.horse/#clients> states:
//
// > Nicknames are non-empty strings with the following restrictions:
// >
// > - They MUST NOT contain any of the following characters: space `(' ',
// >   0x20)`, comma `(',', 0x2C)`, asterisk `('*', 0x2A)`, question mark
// >   `('?', 0x3F)`, exclamation mark `('!', 0x21)`, at sign `('@', 0x40)`.
// >
// > - They MUST NOT start with any of the following characters: dollar `('$',
// >   0x24)`, colon `(':', 0x3A)`.
// >
// > - They MUST NOT start with a character listed as a channel type, channel
// >   membership prefix, or prefix listed in the IRCv3 multi-prefix Extension.
// >
// > - They SHOULD NOT contain any dot character `('.', 0x2E)`.
//
// Notes on item 3:
//
// - Channel type characters are server-defined but commonly include `#` and
//   `&`.
//
// - Channel membership prefixes are server-defined but commonly include `~`,
//   `&`, `@`, `%`, and `+`.
//
// - <https://ircv3.net/specs/extensions/multi-prefix> doesn't seem to specify
//   any additional prefixes.
//
// In addition to the above, in order to be sent in messages, nicknames cannot
// contain NUL, CR, or LF.

use crate::{FinalParam, MedialParam};
use nutype::nutype;
use thiserror::Error;

#[nutype(
    validate(with = validate, error = NicknameError),
    derive(AsRef, Clone, Debug, Deref, Display, Eq, FromStr, Into, PartialEq, TryFrom),
)]
pub struct Nickname(String);

impl PartialEq<str> for Nickname {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl<'a> PartialEq<&'a str> for Nickname {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

fn validate(s: &str) -> Result<(), NicknameError> {
    if s.is_empty() {
        Err(NicknameError::Empty)
    } else if s.starts_with(['$', ':', '#', '&', '~', '@', '%', '+']) {
        Err(NicknameError::BadStart)
    } else if s.contains(['\0', '\r', '\n', ' ', ',', '*', '?', '!', '@']) {
        Err(NicknameError::BadCharacter)
    } else {
        Ok(())
    }
}

impl From<Nickname> for MedialParam {
    fn from(value: Nickname) -> MedialParam {
        MedialParam::try_from(value.into_inner()).expect("Nickname should be valid MedialParam")
    }
}

impl From<Nickname> for FinalParam {
    fn from(value: Nickname) -> FinalParam {
        FinalParam::from(MedialParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum NicknameError {
    #[error("nicknames cannot be empty")]
    Empty,
    #[error("nicknames cannot start with $, :, #, &, ~, @, %, or +")]
    BadStart,
    #[error("nicknames cannot contain NUL, CR, LF, space, comma, *, ?, !, or @")]
    BadCharacter,
}
