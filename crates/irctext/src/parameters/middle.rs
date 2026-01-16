use thiserror::Error;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MiddleParam(String);

validstr!(MiddleParam, ParseMiddleParamError, validate);
strserde!(MiddleParam, "an IRC middle parameter");

fn validate(s: &str) -> Result<(), ParseMiddleParamError> {
    if s.is_empty() {
        Err(ParseMiddleParamError::Empty)
    } else if s.starts_with(':') {
        Err(ParseMiddleParamError::StartsWithColon)
    } else if s.contains(['\0', '\r', '\n', ' ']) {
        Err(ParseMiddleParamError::BadCharacter)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum ParseMiddleParamError {
    #[error("middle parameters cannot be empty")]
    Empty,
    #[error("middle parameters cannot start with a colon")]
    StartsWithColon,
    #[error("middle parameters cannot contain NUL, CR, LF, or SPACE")]
    BadCharacter,
}
