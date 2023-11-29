use super::verb::Verb;
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Command<'a> {
    Verb(Verb<'a>),
    Reply(u16),
}

impl fmt::Display for Command<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Verb(name) => write!(f, "{name}"),
            Command::Reply(code) => write!(f, "{code:03}"),
        }
    }
}

impl<'a> TryFrom<&'a str> for Command<'a> {
    type Error = CommandError;

    fn try_from(s: &'a str) -> Result<Command<'a>, CommandError> {
        if let Ok(name) = Verb::try_from(s) {
            Ok(Command::Verb(name))
        } else if s.len() == 3 && s.chars().all(|ch| ch.is_ascii_digit()) {
            let code = s
                .parse::<u16>()
                .expect("Three-digit number should be valid u16");
            Ok(Command::Reply(code))
        } else {
            Err(CommandError)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("invalid command")]
pub(crate) struct CommandError;
