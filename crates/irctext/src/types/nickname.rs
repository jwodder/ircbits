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

use crate::types::{ModeTarget, MsgTarget, ReplyTarget};
use crate::{CaseMapping, FinalParam, MedialParam};
use std::borrow::Cow;
use thiserror::Error;

#[derive(Clone, Eq, PartialEq)]
pub struct Nickname(String);

validstr!(Nickname, ParseNicknameError, validate);
strserde!(Nickname, "an IRC nickname");

fn validate(s: &str) -> Result<(), ParseNicknameError> {
    if s.is_empty() {
        Err(ParseNicknameError::Empty)
    } else if s.starts_with(['$', ':', '#', '&', '~', '@', '%', '+']) {
        Err(ParseNicknameError::BadStart)
    } else if s.contains(['\0', '\r', '\n', ' ', ',', '*', '?', '!', '@']) {
        Err(ParseNicknameError::BadCharacter)
    } else {
        Ok(())
    }
}

impl Nickname {
    #[expect(clippy::missing_panics_doc)]
    pub fn to_lowercase(&self, cm: CaseMapping) -> Nickname {
        Nickname::try_from(cm.lowercase_str(self.as_str()).into_owned())
            .expect("Case-mapped nickname should still be valid")
    }

    #[expect(clippy::missing_panics_doc)]
    pub fn into_lowercase(self, cm: CaseMapping) -> Nickname {
        match cm.lowercase_str(self.as_str()) {
            Cow::Borrowed(_) => self,
            Cow::Owned(s) => {
                Nickname::try_from(s).expect("Case-mapped nickname should still be valid")
            }
        }
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

impl PartialEq<ModeTarget> for Nickname {
    fn eq(&self, other: &ModeTarget) -> bool {
        matches!(other, ModeTarget::Nick(nick) if nick == self)
    }
}

impl PartialEq<MsgTarget> for Nickname {
    fn eq(&self, other: &MsgTarget) -> bool {
        matches!(other, MsgTarget::Nick(nick) if nick == self)
    }
}

impl PartialEq<ReplyTarget> for Nickname {
    fn eq(&self, other: &ReplyTarget) -> bool {
        matches!(other, ReplyTarget::Nick(nick) if nick == self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseNicknameError {
    #[error("nicknames cannot be empty")]
    Empty,
    #[error("nicknames cannot start with $, :, #, &, ~, @, %, or +")]
    BadStart,
    #[error("nicknames cannot contain NUL, CR, LF, space, comma, *, ?, !, or @")]
    BadCharacter,
}
