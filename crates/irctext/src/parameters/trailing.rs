use super::MiddleParam;
use thiserror::Error;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TrailingParam(String);

validstr!(TrailingParam, ParseTrailingParamError, validate);
strserde!(TrailingParam, "an IRC trailing parameter");

impl From<MiddleParam> for TrailingParam {
    fn from(value: MiddleParam) -> TrailingParam {
        TrailingParam(value.into_inner())
    }
}

fn validate(s: &str) -> Result<(), ParseTrailingParamError> {
    if s.contains(['\0', '\r', '\n']) {
        Err(ParseTrailingParamError)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
#[error("parameters cannot contain NUL, CR, or LF")]
pub struct ParseTrailingParamError;
