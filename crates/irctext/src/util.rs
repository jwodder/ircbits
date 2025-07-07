use crate::ClientMessageError;
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

pub(crate) fn split_param<T>(s: &str) -> Result<Vec<T>, ClientMessageError>
where
    T: TryFrom<String>,
    <T as TryFrom<String>>::Error: Into<ClientMessageError>,
{
    s.split(',')
        .map(|s| T::try_from(s.to_owned()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
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
