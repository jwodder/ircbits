use super::{FinalParam, FinalParamError, MedialParam, MedialParamError, ParamRef};
use crate::util::split_word;
use thiserror::Error;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ParameterList {
    medial: Vec<MedialParam>,
    finalp: Option<FinalParam>,
}

impl ParameterList {
    pub fn builder() -> ParameterListBuilder {
        ParameterListBuilder::new()
    }

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

impl TryFrom<ParameterList> for (FinalParam,) {
    type Error = ParameterListSizeError;

    fn try_from(mut params: ParameterList) -> Result<(FinalParam,), ParameterListSizeError> {
        if params.len() == 1 {
            let p = params
                .medial
                .pop()
                .map(FinalParam::from)
                .or(params.finalp)
                .expect("There should be something to unwrap when len is 1");
            Ok((p,))
        } else {
            Err(ParameterListSizeError {
                requested: 1,
                received: params.len(),
            })
        }
    }
}

impl TryFrom<ParameterList> for (MedialParam, MedialParam, MedialParam, FinalParam) {
    type Error = ParameterListSizeError;

    fn try_from(
        params: ParameterList,
    ) -> Result<(MedialParam, MedialParam, MedialParam, FinalParam), ParameterListSizeError> {
        if params.len() == 4 {
            let mut medials = params.medial.into_iter();
            let p1 = medials
                .next()
                .expect("First element should exist when len is 4");
            let p2 = medials
                .next()
                .expect("Second element should exist when len is 4");
            let p3 = medials
                .next()
                .expect("Third element should exist when len is 4");
            let p4 = medials
                .next()
                .map(FinalParam::from)
                .or(params.finalp)
                .expect("Fourth element should exist when len is 4");
            Ok((p1, p2, p3, p4))
        } else {
            Err(ParameterListSizeError {
                requested: 4,
                received: params.len(),
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParameterListError {
    #[error(transparent)]
    Medial(#[from] MedialParamError),
    #[error(transparent)]
    Final(#[from] FinalParamError),
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("invalid number of parameters: expected {requested}, received {received}")]
pub struct ParameterListSizeError {
    requested: usize,
    received: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ParameterListBuilder(ParameterList);

impl ParameterListBuilder {
    pub fn new() -> ParameterListBuilder {
        ParameterListBuilder::default()
    }

    pub fn push_medial<P: Into<MedialParam>>(&mut self, param: P) {
        self.0.medial.push(param.into());
    }

    pub fn with_medial<P: Into<MedialParam>>(mut self, param: P) -> Self {
        self.push_medial(param);
        self
    }

    pub fn with_final<P: Into<FinalParam>>(mut self, param: P) -> ParameterList {
        self.0.finalp = Some(param.into());
        self.0
    }

    pub fn finish(self) -> ParameterList {
        self.0
    }
}
