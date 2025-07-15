// <https://modern.ircdocs.horse/#channels>:
//
// > Apart from the requirement of the first character being a valid channel
// > type prefix character; the only restriction on a channel name is that it
// > may not contain any spaces `(' ', 0x20)`, a control G / `BELL` `('^G',
// > 0x07)`, or a comma `(',', 0x2C)` (which is used as a list item separator
// > by the protocol).
//
// Note that the set of valid channel type prefixes varies from server to
// server, but for now, to keep things simple, this library treats '#' and '&'
// — and only those characters — as channel type prefixes.
use crate::types::{ModeTarget, MsgTarget};
use crate::{CaseMapping, FinalParam, MedialParam};
use std::borrow::Cow;
use thiserror::Error;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Channel(String);

validstr!(Channel, ParseChannelError, validate);
strserde!(Channel, "an IRC channel name");

fn validate(s: &str) -> Result<(), ParseChannelError> {
    if !channel_prefixed(s) {
        Err(ParseChannelError::BadStart)
    } else if s.contains(['\0', '\r', '\n', ' ', '\x07', ',']) {
        Err(ParseChannelError::BadCharacter)
    } else {
        Ok(())
    }
}

impl Channel {
    #[expect(clippy::missing_panics_doc)]
    pub fn to_lowercase(&self, cm: CaseMapping) -> Channel {
        Channel::try_from(cm.lowercase_str(self.as_str()).into_owned())
            .expect("Case-mapped channel should still be valid")
    }

    #[expect(clippy::missing_panics_doc)]
    pub fn into_lowercase(self, cm: CaseMapping) -> Channel {
        match cm.lowercase_str(self.as_str()) {
            Cow::Borrowed(_) => self,
            Cow::Owned(s) => {
                Channel::try_from(s).expect("Case-mapped channel should still be valid")
            }
        }
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

impl PartialEq<ModeTarget> for Channel {
    fn eq(&self, other: &ModeTarget) -> bool {
        matches!(other, ModeTarget::Channel(chan) if chan == self)
    }
}

impl PartialEq<MsgTarget> for Channel {
    fn eq(&self, other: &MsgTarget) -> bool {
        matches!(other, MsgTarget::Channel(chan) if chan == self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum ParseChannelError {
    #[error("channels must start with '#' or '&'")]
    BadStart,
    #[error("channels cannot contain NUL, CR, LF, SPACE, BELL, or comma")]
    BadCharacter,
}

/// Returns `true` if `s` starts with one of the channel type prefixes
/// recognized by this library
pub(crate) fn channel_prefixed(s: &str) -> bool {
    s.starts_with(crate::CHANNEL_PREFIXES)
}
