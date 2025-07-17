use crate::types::ChannelMembership;
use crate::util::pop_channel_membership;
use crate::{FinalParam, MedialParam, TryFromStringError};
use std::fmt;
use thiserror::Error;

const IS_AWAY: char = 'G';
const NOT_AWAY: char = 'H';

/// The "flags" portion of `RPL_WHOREPLY`
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WhoFlags {
    pub is_away: bool,
    pub is_op: bool,
    pub channel_membership: Option<ChannelMembership>,
    // "user mode characters and other arbitrary server-specific flags"
    pub flags: String,
}

impl fmt::Display for WhoFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", if self.is_away { IS_AWAY } else { NOT_AWAY })?;
        if self.is_op {
            write!(f, "*")?;
        }
        if let Some(cm) = self.channel_membership {
            write!(f, "{}", cm.as_prefix())?;
        }
        write!(f, "{}", self.flags)?;
        Ok(())
    }
}

impl std::str::FromStr for WhoFlags {
    type Err = ParseWhoFlagsError;

    fn from_str(s: &str) -> Result<WhoFlags, ParseWhoFlagsError> {
        let (is_away, s) = if let Some(rest) = s.strip_prefix(IS_AWAY) {
            (true, rest)
        } else if let Some(rest) = s.strip_prefix(NOT_AWAY) {
            (false, rest)
        } else {
            return Err(ParseWhoFlagsError::NoIsAway);
        };
        let (is_op, s) = if let Some(rest) = s.strip_prefix('*') {
            (true, rest)
        } else {
            (false, s)
        };
        let (channel_membership, flags) = pop_channel_membership(s);
        Ok(WhoFlags {
            is_away,
            is_op,
            channel_membership,
            flags: flags.to_owned(),
        })
    }
}

impl TryFrom<String> for WhoFlags {
    type Error = TryFromStringError<ParseWhoFlagsError>;

    fn try_from(string: String) -> Result<WhoFlags, TryFromStringError<ParseWhoFlagsError>> {
        match string.parse() {
            Ok(src) => Ok(src),
            Err(inner) => Err(TryFromStringError { inner, string }),
        }
    }
}

impl From<WhoFlags> for MedialParam {
    fn from(value: WhoFlags) -> MedialParam {
        MedialParam::try_from(value.to_string()).expect("WhoFlags should be valid MedialParam")
    }
}

impl From<WhoFlags> for FinalParam {
    fn from(value: WhoFlags) -> FinalParam {
        FinalParam::from(MedialParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum ParseWhoFlagsError {
    #[error("isaway (H/G) token missing")]
    NoIsAway,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn here() {
        let flags = "H".parse::<WhoFlags>().unwrap();
        assert_eq!(
            flags,
            WhoFlags {
                is_away: false,
                is_op: false,
                channel_membership: None,
                flags: String::new(),
            }
        );
    }
}
