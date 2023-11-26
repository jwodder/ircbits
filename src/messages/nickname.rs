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

use std::borrow::Cow;
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Nickname<'a>(Cow<'a, str>);

impl fmt::Display for Nickname<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<str> for Nickname<'_> {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl<'a> PartialEq<&'a str> for Nickname<'_> {
    fn eq(&self, other: &&'a str) -> bool {
        &self.0 == other
    }
}

impl AsRef<str> for Nickname<'_> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'a> TryFrom<&'a str> for Nickname<'a> {
    type Error = NicknameError;

    fn try_from(s: &'a str) -> Result<Nickname<'a>, NicknameError> {
        if s.is_empty() {
            Err(NicknameError::Empty)
        } else if s.starts_with(['$', ':', '#', '&', '~', '@', '%', '+']) {
            Err(NicknameError::BadStart)
        } else if s.contains(['\0', '\r', '\n', ' ', ',', '*', '?', '!', '@']) {
            Err(NicknameError::BadCharacter)
        } else {
            Ok(Nickname(s.into()))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum NicknameError {
    #[error("nicknames cannot be empty")]
    Empty,
    #[error("nicknames cannot start with $, :, #, &, ~, @, %, or +")]
    BadStart,
    #[error("nicknames cannot contain NUL, CR, LF, space, comma, *, ?, !, or @")]
    BadCharacter,
}
