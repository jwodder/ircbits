use super::command_name::CommandName;
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RawCommand<'a> {
    Command(CommandName<'a>),
    Reply(u16),
}

impl fmt::Display for RawCommand<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RawCommand::Command(name) => write!(f, "{name}"),
            RawCommand::Reply(code) => write!(f, "{code:03}"),
        }
    }
}

impl<'a> TryFrom<&'a str> for RawCommand<'a> {
    type Error = RawCommandError;

    fn try_from(s: &'a str) -> Result<RawCommand<'a>, RawCommandError> {
        if let Ok(name) = CommandName::try_from(s) {
            Ok(RawCommand::Command(name))
        } else if s.len() == 3 && s.chars().all(|ch| ch.is_ascii_digit()) {
            let code = s
                .parse::<u16>()
                .expect("Three-digit number should be valid u16");
            Ok(RawCommand::Reply(code))
        } else {
            Err(RawCommandError)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("invalid command")]
pub(crate) struct RawCommandError;
