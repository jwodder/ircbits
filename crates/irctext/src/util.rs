use crate::{Channel, ClientMessageError, Key, Target};
use std::fmt;

pub(crate) fn split_word(s: &str) -> (&str, &str) {
    match s.split_once(' ') {
        Some((s1, s2)) => (s1, s2.trim_start_matches(' ')),
        None => (s, ""),
    }
}

pub(crate) fn join_with_commas<I>(iter: I) -> String
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut s = String::new();
    let mut first = true;
    for item in iter {
        if !std::mem::replace(&mut first, false) {
            s.push(',');
        }
        s.push_str(item.as_ref());
    }
    s
}

pub(crate) fn split_channels(s: String) -> Result<Vec<Channel>, ClientMessageError> {
    match s
        .split(',')
        .map(|s| Channel::try_from(s.to_owned()))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(channels) => Ok(channels),
        Err(source) => Err(ClientMessageError::ParseParam(Box::new(source))),
    }
}

pub(crate) fn split_keys(s: String) -> Result<Vec<Key>, ClientMessageError> {
    match s
        .split(',')
        .map(|s| Key::try_from(s.to_owned()))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(keys) => Ok(keys),
        Err(source) => Err(ClientMessageError::ParseParam(Box::new(source))),
    }
}

pub(crate) fn split_targets(s: String) -> Result<Vec<Target>, ClientMessageError> {
    match s
        .as_str()
        .split(',')
        .map(|s| Target::try_from(s.to_owned()))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(targets) => Ok(targets),
        Err(source) => Err(ClientMessageError::ParseParam(Box::new(source))),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DisplayMaybeFinal<T>(pub Option<T>);

impl<T: fmt::Display> fmt::Display for DisplayMaybeFinal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref value) = self.0 {
            write!(f, " :{value}")
        } else {
            Ok(())
        }
    }
}
