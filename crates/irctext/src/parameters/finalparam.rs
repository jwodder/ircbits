use super::MedialParam;
use thiserror::Error;

#[derive(Clone, Eq, PartialEq)]
pub struct FinalParam(String);

validstr!(FinalParam, ParseFinalParamError, validate);
strserde!(FinalParam, "an IRC trailing parameter");

impl From<MedialParam> for FinalParam {
    fn from(value: MedialParam) -> FinalParam {
        FinalParam(value.into_inner())
    }
}

fn validate(s: &str) -> Result<(), ParseFinalParamError> {
    if s.contains(['\0', '\r', '\n']) {
        Err(ParseFinalParamError)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("parameters cannot contain NUL, CR, or LF")]
pub struct ParseFinalParamError;
