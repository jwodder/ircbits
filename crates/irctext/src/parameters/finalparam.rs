use super::MedialParam;
use nutype::nutype;
use thiserror::Error;

#[nutype(
    validate(with = validate, error = FinalParamError),
    derive(AsRef, Clone, Debug, Deref, Display, Eq, FromStr, Into, PartialEq, TryFrom),
)]
pub struct FinalParam(String);

impl PartialEq<str> for FinalParam {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl<'a> PartialEq<&'a str> for FinalParam {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

impl From<MedialParam> for FinalParam {
    fn from(value: MedialParam) -> FinalParam {
        FinalParam::try_from(value.into_inner()).expect("MedialParam should be valid FinalParam")
    }
}

fn validate(s: &str) -> Result<(), FinalParamError> {
    if s.contains(['\0', '\r', '\n']) {
        Err(FinalParamError)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("parameters cannot contain NUL, CR, or LF")]
pub struct FinalParamError;
