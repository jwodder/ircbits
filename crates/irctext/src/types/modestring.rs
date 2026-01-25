use crate::{MiddleParam, TrailingParam};
use thiserror::Error;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModeString(String);

validstr!(ModeString, ParseModeStringError, validate);

fn validate(s: &str) -> Result<(), ParseModeStringError> {
    if !s.starts_with(['+', '-']) {
        Err(ParseModeStringError::BadStart)
    } else if s.contains(|c: char| !(c.is_ascii_alphabetic() || c == '+' || c == '-')) {
        Err(ParseModeStringError::BadCharacter)
    } else {
        Ok(())
    }
}

impl ModeString {
    /// Returns `true` if the mode string does not enable or disable any modes
    pub fn is_nil(&self) -> bool {
        self.0.chars().all(|ch| ch == '+' || ch == '-')
    }

    pub fn modes(&self) -> Modes<'_> {
        Modes::new(self)
    }

    pub fn state(&self, mode: char) -> Option<ModeState> {
        self.modes()
            .filter_map(|(st, ch)| (ch == mode).then_some(st))
            .last()
    }
}

impl From<ModeString> for MiddleParam {
    fn from(value: ModeString) -> MiddleParam {
        MiddleParam::try_from(value.into_inner()).expect("Mode string should be valid MiddleParam")
    }
}

impl From<ModeString> for TrailingParam {
    fn from(value: ModeString) -> TrailingParam {
        TrailingParam::from(MiddleParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum ParseModeStringError {
    #[error("mode strings must start with + or -")]
    BadStart,
    #[error("mode strings can only contain +, -, and ASCII letters")]
    BadCharacter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModeState {
    Enabled,
    Disabled,
}

#[derive(Clone, Debug)]
pub struct Modes<'a> {
    inner: std::str::Chars<'a>,
    state: ModeState,
}

impl<'a> Modes<'a> {
    fn new(ms: &'a ModeString) -> Modes<'a> {
        Modes {
            inner: ms.0.chars(),
            state: ModeState::Enabled,
        }
    }
}

impl Iterator for Modes<'_> {
    type Item = (ModeState, char);

    fn next(&mut self) -> Option<(ModeState, char)> {
        loop {
            match self.inner.next()? {
                '+' => self.state = ModeState::Enabled,
                '-' => self.state = ModeState::Disabled,
                m => return Some((self.state, m)),
            }
        }
    }
}

impl std::iter::FusedIterator for Modes<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("+")]
    #[case("-")]
    #[case("+-")]
    #[case("+-+")]
    #[case("--")]
    fn nil(#[case] ms: ModeString) {
        assert!(ms.is_nil());
        let mut iter = ms.modes();
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(ms.state('x'), None);
    }

    #[test]
    fn all_enabled() {
        let ms = "+Ziw".parse::<ModeString>().unwrap();
        assert!(!ms.is_nil());
        let mut iter = ms.modes();
        assert_eq!(iter.next(), Some((ModeState::Enabled, 'Z')));
        assert_eq!(iter.next(), Some((ModeState::Enabled, 'i')));
        assert_eq!(iter.next(), Some((ModeState::Enabled, 'w')));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(ms.state('Z'), Some(ModeState::Enabled));
        assert_eq!(ms.state('i'), Some(ModeState::Enabled));
        assert_eq!(ms.state('w'), Some(ModeState::Enabled));
        assert_eq!(ms.state('x'), None);
    }

    #[test]
    fn all_disabled() {
        let ms = "-Ziw".parse::<ModeString>().unwrap();
        assert!(!ms.is_nil());
        let mut iter = ms.modes();
        assert_eq!(iter.next(), Some((ModeState::Disabled, 'Z')));
        assert_eq!(iter.next(), Some((ModeState::Disabled, 'i')));
        assert_eq!(iter.next(), Some((ModeState::Disabled, 'w')));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(ms.state('Z'), Some(ModeState::Disabled));
        assert_eq!(ms.state('i'), Some(ModeState::Disabled));
        assert_eq!(ms.state('w'), Some(ModeState::Disabled));
        assert_eq!(ms.state('x'), None);
    }

    #[test]
    fn mixed() {
        let ms = "+Zi-iw".parse::<ModeString>().unwrap();
        assert!(!ms.is_nil());
        let mut iter = ms.modes();
        assert_eq!(iter.next(), Some((ModeState::Enabled, 'Z')));
        assert_eq!(iter.next(), Some((ModeState::Enabled, 'i')));
        assert_eq!(iter.next(), Some((ModeState::Disabled, 'i')));
        assert_eq!(iter.next(), Some((ModeState::Disabled, 'w')));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(ms.state('Z'), Some(ModeState::Enabled));
        assert_eq!(ms.state('i'), Some(ModeState::Disabled));
        assert_eq!(ms.state('w'), Some(ModeState::Disabled));
        assert_eq!(ms.state('x'), None);
    }
}
