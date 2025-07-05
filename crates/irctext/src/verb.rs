use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Verb(String);

common_string!(Verb, VerbError);

impl TryFrom<String> for Verb {
    type Error = VerbError;

    fn try_from(s: String) -> Result<Verb, VerbError> {
        if s.is_empty() {
            Err(VerbError::Empty)
        } else if s.contains(|ch: char| !ch.is_ascii_alphabetic()) {
            Err(VerbError::BadCharacter)
        } else {
            Ok(Verb(s))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum VerbError {
    #[error("verbs cannot be empty")]
    Empty,
    #[error("verbs may only contain letters")]
    BadCharacter,
}
