use thiserror::Error;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MedialParam(String);

validstr!(MedialParam, ParseMedialParamError, validate);
strserde!(MedialParam, "an IRC middle parameter");

fn validate(s: &str) -> Result<(), ParseMedialParamError> {
    if s.is_empty() {
        Err(ParseMedialParamError::Empty)
    } else if s.starts_with(':') {
        Err(ParseMedialParamError::StartsWithColon)
    } else if s.contains(['\0', '\r', '\n', ' ']) {
        Err(ParseMedialParamError::BadCharacter)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum ParseMedialParamError {
    #[error("medial parameters cannot be empty")]
    Empty,
    #[error("medial parameters cannot start with a colon")]
    StartsWithColon,
    #[error("medial parameters cannot contain NUL, CR, LF, or SPACE")]
    BadCharacter,
}
