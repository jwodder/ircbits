use super::{FinalParam, FinalParamError, MedialParam, MedialParamError, ParamRef};
use crate::util::split_word;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterList {
    medial: Vec<MedialParam>,
    finalp: Option<FinalParam>,
}

impl ParameterList {
    pub fn len(&self) -> usize {
        self.medial
            .len()
            .saturating_add(usize::from(self.finalp.is_some()))
    }

    pub fn is_empty(&self) -> bool {
        self.medial.is_empty() && self.finalp.is_none()
    }

    pub fn get(&self, index: usize) -> Option<ParamRef<'_>> {
        if index < self.medial.len() {
            self.medial.get(index).map(ParamRef::Medial)
        } else if index == self.medial.len() {
            self.finalp.as_ref().map(ParamRef::Final)
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = ParamRef<'_>> + '_ {
        self.medial
            .iter()
            .map(ParamRef::Medial)
            .chain(self.finalp.as_ref().map(ParamRef::Final))
    }
}

impl<const N: usize> PartialEq<[&str; N]> for ParameterList {
    fn eq(&self, other: &[&str; N]) -> bool {
        N == self.len() && std::iter::zip(self.iter(), other).all(|(param, &s)| param == s)
    }
}

impl<const N: usize> PartialEq<[&str; N]> for &ParameterList {
    fn eq(&self, other: &[&str; N]) -> bool {
        *self == other
    }
}

impl std::str::FromStr for ParameterList {
    type Err = ParameterListError;

    fn from_str(mut s: &str) -> Result<ParameterList, ParameterListError> {
        let mut medial = Vec::new();
        let mut finalp = None;
        while !s.is_empty() {
            if let Some(trail) = s.strip_prefix(':') {
                finalp = Some(trail.parse::<FinalParam>()?);
                s = "";
            } else {
                let (param, rest) = split_word(s);
                medial.push(param.parse::<MedialParam>()?);
                s = rest;
            }
        }
        Ok(ParameterList { medial, finalp })
    }
}

impl TryFrom<String> for ParameterList {
    type Error = ParameterListError;

    fn try_from(s: String) -> Result<ParameterList, ParameterListError> {
        s.parse::<ParameterList>()
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParameterListError {
    #[error(transparent)]
    Medial(#[from] MedialParamError),
    #[error(transparent)]
    Final(#[from] FinalParamError),
}
