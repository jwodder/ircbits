use super::verb::Verb;
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Command {
    Verb(Verb),
    Reply(u16),
}

impl Command {
    pub fn is_verb(&self) -> bool {
        matches!(self, Command::Verb(_))
    }

    pub fn is_reply(&self) -> bool {
        matches!(self, Command::Reply(_))
    }
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
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Command, ParseCommandError> {
        String::from(s).try_into()
    }
}

impl TryFrom<String> for Command {
    type Error = ParseCommandError;

    fn try_from(s: String) -> Result<Command, ParseCommandError> {
        if s.len() == 3 && s.chars().all(|ch| ch.is_ascii_digit()) {
            let code = s
                .parse::<u16>()
                .expect("Three-digit number should be valid u16");
            Ok(Command::Reply(code))
        } else {
            Ok(Command::Verb(Verb::from(s)))
        }
    }
}

impl From<Verb> for Command {
    fn from(value: Verb) -> Command {
        Command::Verb(value)
    }
}

impl From<u16> for Command {
    fn from(value: u16) -> Command {
        Command::Reply(value)
    }
}

impl PartialEq<Verb> for Command {
    fn eq(&self, other: &Verb) -> bool {
        matches!(self, Command::Verb(v) if v == other)
    }
}

impl PartialEq<u16> for Command {
    fn eq(&self, other: &u16) -> bool {
        *self == Command::Reply(*other)
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
#[error("invalid command")]
pub struct ParseCommandError;
