use crate::{FinalParam, MedialParam, TryFromStringError};
use std::fmt;
use thiserror::Error;

/// Channel status used in `RPL_NAMREPLY`
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ChannelStatus {
    Public,
    Secret,
    Private,
}

impl ChannelStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChannelStatus::Public => "=",
            ChannelStatus::Secret => "@",
            ChannelStatus::Private => "*",
        }
    }

    pub fn as_char(&self) -> char {
        match self {
            ChannelStatus::Public => '=',
            ChannelStatus::Secret => '@',
            ChannelStatus::Private => '*',
        }
    }
}

impl fmt::Display for ChannelStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str())
    }
}

impl std::str::FromStr for ChannelStatus {
    type Err = ParseChannelStatusError;

    fn from_str(s: &str) -> Result<ChannelStatus, ParseChannelStatusError> {
        match s {
            "=" => Ok(ChannelStatus::Public),
            "@" => Ok(ChannelStatus::Secret),
            "*" => Ok(ChannelStatus::Private),
            _ => Err(ParseChannelStatusError),
        }
    }
}

impl AsRef<str> for ChannelStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl TryFrom<String> for ChannelStatus {
    type Error = TryFromStringError<ParseChannelStatusError>;

    fn try_from(
        string: String,
    ) -> Result<ChannelStatus, TryFromStringError<ParseChannelStatusError>> {
        match string.parse() {
            Ok(src) => Ok(src),
            Err(inner) => Err(TryFromStringError { inner, string }),
        }
    }
}

impl From<ChannelStatus> for MedialParam {
    fn from(value: ChannelStatus) -> MedialParam {
        value
            .as_str()
            .parse::<MedialParam>()
            .expect("ChannelStatus should be a valid MedialParam")
    }
}

impl From<ChannelStatus> for FinalParam {
    fn from(value: ChannelStatus) -> FinalParam {
        FinalParam::from(MedialParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("invalid/unrecognized channel status string")]
pub struct ParseChannelStatusError;
