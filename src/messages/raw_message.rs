use super::parameter::{Parameter, ParameterError};
use super::raw_command::{RawCommand, RawCommandError};
use super::source::{Source, SourceError};
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RawMessage<'a> {
    source: Option<Source<'a>>,
    command: RawCommand<'a>,
    parameters: Vec<Parameter<'a>>,
}

impl<'a> RawMessage<'a> {
    pub(crate) fn source(&self) -> Option<&Source<'a>> {
        self.source.as_ref()
    }

    pub(crate) fn command(&self) -> &RawCommand<'a> {
        &self.command
    }

    pub(crate) fn parameters(&self) -> &[Parameter<'a>] {
        &self.parameters
    }
}

impl fmt::Display for RawMessage<'_> {
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

impl<'a> TryFrom<&'a str> for RawMessage<'a> {
    type Error = RawMessageError;

    // Assumes `s` does not contain the terminating CR LF
    fn try_from(mut s: &'a str) -> Result<RawMessage<'a>, RawMessageError> {
        let source = if let Some(s2) = s.strip_prefix(':') {
            let (source_str, rest) = split_word(s2);
            s = rest;
            Some(Source::try_from(source_str)?)
        } else {
            None
        };
        let (cmd_str, mut s) = split_word(s);
        let command = RawCommand::try_from(cmd_str)?;
        let mut parameters = Vec::new();
        while !s.is_empty() {
            if let Some(trail) = s.strip_prefix(':') {
                parameters.push(Parameter::try_from(trail)?);
                s = "";
            } else {
                let (param, rest) = split_word(s);
                parameters.push(Parameter::try_from(param)?);
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
    Command(#[from] RawCommandError),
    #[error("invalid parameter")]
    Parameter(#[from] ParameterError),
}

fn split_word(s: &str) -> (&str, &str) {
    match s.split_once(' ') {
        Some((s1, s2)) => (s1, s2.trim_start_matches(' ')),
        None => (s, ""),
    }
}
