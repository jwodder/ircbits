use crate::types::{Nickname, ParseNicknameError};
use crate::{FinalParam, MedialParam, TryFromStringError};
use std::fmt;
use thiserror::Error;

const IS_AWAY: char = '-';
const NOT_AWAY: char = '+';

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserHostReply {
    pub nickname: Nickname,
    pub is_op: bool,
    pub is_away: bool,
    // Note: On libera.chat, the <hostname> portion is actually
    // [~]<user>@<host>
    pub hostname: String,
}

impl fmt::Display for UserHostReply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.nickname)?;
        if self.is_op {
            write!(f, "*")?;
        }
        write!(
            f,
            "={}{}",
            if self.is_away { IS_AWAY } else { NOT_AWAY },
            self.hostname
        )?;
        Ok(())
    }
}

impl std::str::FromStr for UserHostReply {
    type Err = ParseUserHostReplyError;

    fn from_str(s: &str) -> Result<UserHostReply, ParseUserHostReplyError> {
        let (left, right) = s.split_once('=').ok_or(ParseUserHostReplyError::NoEq)?;
        let (rawnick, is_op) = if let Some(rawnick) = left.strip_suffix('*') {
            (rawnick, true)
        } else {
            (left, false)
        };
        let nickname = rawnick.parse::<Nickname>()?;
        let (is_away, hostname) = if let Some(host) = right.strip_prefix(IS_AWAY) {
            (true, host.to_owned())
        } else if let Some(host) = right.strip_prefix(NOT_AWAY) {
            (false, host.to_owned())
        } else {
            return Err(ParseUserHostReplyError::NoIsAway);
        };
        Ok(UserHostReply {
            nickname,
            is_op,
            is_away,
            hostname,
        })
    }
}

impl TryFrom<String> for UserHostReply {
    type Error = TryFromStringError<ParseUserHostReplyError>;

    fn try_from(
        string: String,
    ) -> Result<UserHostReply, TryFromStringError<ParseUserHostReplyError>> {
        match string.parse() {
            Ok(src) => Ok(src),
            Err(inner) => Err(TryFromStringError { inner, string }),
        }
    }
}

impl From<UserHostReply> for MedialParam {
    fn from(value: UserHostReply) -> MedialParam {
        MedialParam::try_from(value.to_string())
            .expect("USERHOST reply should be valid MedialParam")
    }
}

impl From<UserHostReply> for FinalParam {
    fn from(value: UserHostReply) -> FinalParam {
        FinalParam::from(MedialParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseUserHostReplyError {
    #[error("equals sign missing")]
    NoEq,
    #[error("invalid nickname component")]
    Nickname(#[from] ParseNicknameError),
    #[error("isaway (+/-) token missing")]
    NoIsAway,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_op_not_away() {
        let uhr = "jwodder=+~jwuser@127.0.0.1"
            .parse::<UserHostReply>()
            .unwrap();
        assert_eq!(
            uhr,
            UserHostReply {
                nickname: "jwodder".parse::<Nickname>().unwrap(),
                is_op: false,
                is_away: false,
                hostname: String::from("~jwuser@127.0.0.1")
            }
        );
    }
}
