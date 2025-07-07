use super::{
    FinalParam, MedialParam, ParamRef, Parameter, ParseFinalParamError, ParseMedialParamError,
};
use crate::util::split_word;
use std::cmp::Ordering;
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
        match index.cmp(&self.medial.len()) {
            Ordering::Less => self.medial.get(index).map(ParamRef::Medial),
            Ordering::Equal => self.finalp.as_ref().map(ParamRef::Final),
            Ordering::Greater => None,
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

impl IntoIterator for ParameterList {
    type IntoIter = ParameterListIntoIter;
    type Item = Parameter;

    fn into_iter(self) -> ParameterListIntoIter {
        ParameterListIntoIter::new(self)
    }
}

impl std::str::FromStr for ParameterList {
    type Err = ParseParameterListError;

    fn from_str(mut s: &str) -> Result<ParameterList, ParseParameterListError> {
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
    type Error = ParseParameterListError;

    fn try_from(s: String) -> Result<ParameterList, ParseParameterListError> {
        s.parse::<ParameterList>()
    }
}

impl TryFrom<ParameterList> for () {
    type Error = ParameterListSizeError;

    fn try_from(params: ParameterList) -> Result<(), ParameterListSizeError> {
        if params.is_empty() {
            Ok(())
        } else {
            Err(ParameterListSizeError::Exact {
                requested: 0,
                received: params.len(),
            })
        }
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
            Err(ParameterListSizeError::Exact {
                requested: 1,
                received: params.len(),
            })
        }
    }
}

impl TryFrom<ParameterList> for (Option<FinalParam>,) {
    type Error = ParameterListSizeError;

    fn try_from(params: ParameterList) -> Result<(Option<FinalParam>,), ParameterListSizeError> {
        match (params.len(), params.finalp.is_some()) {
            (1, false) => Ok((params.medial.into_iter().next().map(FinalParam::from),)),
            (0, _) => Ok((params.finalp,)),
            _ => Err(ParameterListSizeError::Range {
                min_requested: 0,
                max_requested: 1,
                received: params.len(),
            }),
        }
    }
}

impl TryFrom<ParameterList> for (MedialParam, FinalParam) {
    type Error = ParameterListSizeError;

    fn try_from(
        params: ParameterList,
    ) -> Result<(MedialParam, FinalParam), ParameterListSizeError> {
        if params.len() == 2 {
            let mut medials = params.medial.into_iter();
            let p1 = medials
                .next()
                .expect("First element should exist when len is 2");
            let p2 = medials
                .next()
                .map(FinalParam::from)
                .or(params.finalp)
                .expect("Second element should exist when len is 2");
            Ok((p1, p2))
        } else {
            Err(ParameterListSizeError::Exact {
                requested: 2,
                received: params.len(),
            })
        }
    }
}

impl TryFrom<ParameterList> for (MedialParam, Option<FinalParam>) {
    type Error = ParameterListSizeError;

    fn try_from(
        params: ParameterList,
    ) -> Result<(MedialParam, Option<FinalParam>), ParameterListSizeError> {
        match (params.len(), params.finalp.is_some()) {
            (2, false) => {
                let mut medials = params.medial.into_iter();
                let p1 = medials
                    .next()
                    .expect("First element should exist when len is 2");
                let p2 = medials.next().map(FinalParam::from);
                Ok((p1, p2))
            }
            (1, _) => {
                let mut medials = params.medial.into_iter();
                let p1 = medials
                    .next()
                    .expect("First element should exist when len is 1");
                let p2 = params.finalp;
                Ok((p1, p2))
            }
            _ => Err(ParameterListSizeError::Range {
                min_requested: 1,
                max_requested: 2,
                received: params.len(),
            }),
        }
    }
}

impl TryFrom<ParameterList> for (MedialParam, MedialParam, Option<FinalParam>) {
    type Error = ParameterListSizeError;

    fn try_from(
        params: ParameterList,
    ) -> Result<(MedialParam, MedialParam, Option<FinalParam>), ParameterListSizeError> {
        match (params.len(), params.finalp.is_some()) {
            (3, false) => {
                let mut medials = params.medial.into_iter();
                let p1 = medials
                    .next()
                    .expect("First element should exist when len is 3");
                let p2 = medials
                    .next()
                    .expect("Second element should exist when len is 3");
                let p3 = medials.next().map(FinalParam::from);
                Ok((p1, p2, p3))
            }
            (2, _) => {
                let mut medials = params.medial.into_iter();
                let p1 = medials
                    .next()
                    .expect("First element should exist when len is 2");
                let p2 = medials
                    .next()
                    .expect("Second element should exist when len is 2");
                let p3 = params.finalp;
                Ok((p1, p2, p3))
            }
            _ => Err(ParameterListSizeError::Range {
                min_requested: 2,
                max_requested: 3,
                received: params.len(),
            }),
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
            Err(ParameterListSizeError::Exact {
                requested: 4,
                received: params.len(),
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseParameterListError {
    #[error(transparent)]
    Medial(#[from] ParseMedialParamError),
    #[error(transparent)]
    Final(#[from] ParseFinalParamError),
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParameterListSizeError {
    #[error("invalid number of parameters: expected {requested}, received {received}")]
    Exact { requested: usize, received: usize },
    #[error("invalid number of parameters: expected {min_requested}-{max_requested}, received {received}")]
    Range {
        min_requested: usize,
        max_requested: usize,
        received: usize,
    },
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

    pub fn maybe_with_final<P: Into<FinalParam>>(mut self, param: Option<P>) -> ParameterList {
        self.0.finalp = param.map(Into::into);
        self.0
    }

    pub fn with_final<P: Into<FinalParam>>(mut self, param: P) -> ParameterList {
        self.0.finalp = Some(param.into());
        self.0
    }

    pub fn finish(self) -> ParameterList {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct ParameterListIntoIter(std::vec::IntoIter<Parameter>);

impl ParameterListIntoIter {
    fn new(params: ParameterList) -> Self {
        let mut paramvec = params
            .medial
            .into_iter()
            .map(Parameter::Medial)
            .collect::<Vec<_>>();
        paramvec.extend(params.finalp.map(Parameter::Final));
        ParameterListIntoIter(paramvec.into_iter())
    }
}

impl Iterator for ParameterListIntoIter {
    type Item = Parameter;

    fn next(&mut self) -> Option<Parameter> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl DoubleEndedIterator for ParameterListIntoIter {
    fn next_back(&mut self) -> Option<Parameter> {
        self.0.next_back()
    }
}

impl ExactSizeIterator for ParameterListIntoIter {}

impl std::iter::FusedIterator for ParameterListIntoIter {}
