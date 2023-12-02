use super::command::{Command, CommandError};
use super::parameter::{Parameter, ParameterError};
use super::source::{Source, SourceError};
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RawMessage {
    source: Option<Source>,
    command: Command,
    parameters: Vec<Parameter>,
}

impl RawMessage {
    pub(crate) fn source(&self) -> Option<&Source> {
        self.source.as_ref()
    }

    pub(crate) fn command(&self) -> &Command {
        &self.command
    }

    pub(crate) fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }
}

impl fmt::Display for RawMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(source) = self.source.as_ref() {
            write!(f, ":{source} ")?;
        }
        write!(f, "{}", self.command)?;
        for p in &self.parameters {
            if p.is_middle() {
                write!(f, " {p}")?;
            } else {
                write!(f, " :{p}")?;
            }
        }
        Ok(())
    }
}

impl std::str::FromStr for RawMessage {
    type Err = RawMessageError;

    fn from_str(s: &str) -> Result<RawMessage, RawMessageError> {
        String::from(s).try_into()
    }
}

impl TryFrom<String> for RawMessage {
    type Error = RawMessageError;

    // `s` may optionally end with LF, CR LF, or CR.
    fn try_from(s: String) -> Result<RawMessage, RawMessageError> {
        let mut s = s.strip_suffix('\n').unwrap_or(&*s);
        s = s.strip_suffix('\r').unwrap_or(s);
        let source = if let Some(s2) = s.strip_prefix(':') {
            let (source_str, rest) = split_word(s2);
            s = rest;
            Some(source_str.parse::<Source>()?)
        } else {
            None
        };
        let (cmd_str, mut s) = split_word(s);
        let command = cmd_str.parse::<Command>()?;
        let mut parameters = Vec::new();
        while !s.is_empty() {
            if let Some(trail) = s.strip_prefix(':') {
                parameters.push(trail.parse::<Parameter>()?);
                s = "";
            } else {
                let (param, rest) = split_word(s);
                parameters.push(param.parse::<Parameter>()?);
                s = rest;
            }
        }
        Ok(RawMessage {
            source,
            command,
            parameters,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum RawMessageError {
    #[error("invalid source prefix")]
    Source(#[from] SourceError),
    #[error("invalid command")]
    Command(#[from] CommandError),
    #[error("invalid parameter")]
    Parameter(#[from] ParameterError),
}

fn split_word(s: &str) -> (&str, &str) {
    match s.split_once(' ') {
        Some((s1, s2)) => (s1, s2.trim_start_matches(' ')),
        None => (s, ""),
    }
}
