use super::verb::Verb;
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    Verb(Verb),
    Reply(u16),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Verb(name) => write!(f, "{name}"),
            Command::Reply(code) => write!(f, "{code:03}"),
        }
    }
}

impl std::str::FromStr for Command {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Command, CommandError> {
        String::from(s).try_into()
    }
}

impl TryFrom<String> for Command {
    type Error = CommandError;

    fn try_from(s: String) -> Result<Command, CommandError> {
        if s.len() == 3 && s.chars().all(|ch| ch.is_ascii_digit()) {
            let code = s
                .parse::<u16>()
                .expect("Three-digit number should be valid u16");
            Ok(Command::Reply(code))
        } else if let Ok(name) = Verb::try_from(s) {
            Ok(Command::Verb(name))
        } else {
            Err(CommandError)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("invalid command")]
pub struct CommandError;
