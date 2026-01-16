use crate::ClientMessageError;
use crate::types::ChannelMembership;
use std::fmt::{self, Write};

pub(crate) fn split_word(s: &str) -> (&str, &str) {
    let s = s.trim_start_matches(' ');
    match s.split_once(' ') {
        Some((s1, s2)) => (s1, s2.trim_start_matches(' ')),
        None => (s, ""),
    }
}

pub(crate) fn join_with_space<'a, T>(values: &'a [T]) -> JoinWithChar<'a, T> {
    JoinWithChar(' ', values)
}

pub(crate) fn join_with_commas<'a, T>(values: &'a [T]) -> JoinWithChar<'a, T> {
    JoinWithChar(',', values)
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct JoinWithChar<'a, T>(char, &'a [T]);

impl<T: fmt::Display> fmt::Display for JoinWithChar<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for item in self.1 {
            if !std::mem::replace(&mut first, false) {
                f.write_char(self.0)?;
            }
            write!(f, "{item}")?;
        }
        Ok(())
    }
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

pub(crate) fn pop_channel_membership(s: &str) -> (Option<ChannelMembership>, &str) {
    let mut iter = s.chars();
    if let Some(ch) = iter.next()
        && let Some(mem) = ChannelMembership::from_prefix(ch)
    {
        (Some(mem), iter.as_str())
    } else {
        (None, s)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct DisplayMaybeTrailing<T>(pub Option<T>);

impl<T: fmt::Display> fmt::Display for DisplayMaybeTrailing<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref value) = self.0 {
            write!(f, " :{value}")
        } else {
            Ok(())
        }
    }
}

pub(crate) fn split_spaces(s: &str) -> SplitSpaces<'_> {
    SplitSpaces::new(s)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct SplitSpaces<'a>(&'a str);

impl<'a> SplitSpaces<'a> {
    fn new(s: &'a str) -> Self {
        SplitSpaces(s.trim_start_matches(' '))
    }
}

impl<'a> Iterator for SplitSpaces<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.0.is_empty() {
            None
        } else {
            let (s1, s2) = split_word(self.0);
            self.0 = s2;
            Some(s1)
        }
    }
}

impl std::iter::FusedIterator for SplitSpaces<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    mod split_spaces {
        use super::*;

        #[test]
        fn empty() {
            let mut iter = split_spaces("");
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
        }

        #[test]
        fn spaces() {
            let mut iter = split_spaces("   ");
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
        }

        #[test]
        fn one_word() {
            let mut iter = split_spaces("foo");
            assert_eq!(iter.next(), Some("foo"));
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
        }

        #[test]
        fn two_words() {
            let mut iter = split_spaces("foo  bar");
            assert_eq!(iter.next(), Some("foo"));
            assert_eq!(iter.next(), Some("bar"));
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
        }

        #[test]
        fn leading_trailing_spaces() {
            let mut iter = split_spaces(" foo  bar ");
            assert_eq!(iter.next(), Some("foo"));
            assert_eq!(iter.next(), Some("bar"));
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
        }
    }
}
