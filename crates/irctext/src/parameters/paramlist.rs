use super::{
    MiddleParam, ParamRef, Parameter, ParseMiddleParamError, ParseTrailingParamError, TrailingParam,
};
use crate::TryFromStringError;
use crate::util::split_word;
use std::cmp::Ordering;
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct ParameterList {
    middle: Vec<MiddleParam>,
    trailing: Option<TrailingParam>,
}

impl ParameterList {
    pub fn builder() -> ParameterListBuilder {
        ParameterListBuilder::new()
    }

    pub fn len(&self) -> usize {
        self.middle
            .len()
            .saturating_add(usize::from(self.trailing.is_some()))
    }

    pub fn is_empty(&self) -> bool {
        self.middle.is_empty() && self.trailing.is_none()
    }

    pub fn get(&self, index: usize) -> Option<ParamRef<'_>> {
        match index.cmp(&self.middle.len()) {
            Ordering::Less => self.middle.get(index).map(ParamRef::Middle),
            Ordering::Equal => self.trailing.as_ref().map(ParamRef::Trailing),
            Ordering::Greater => None,
        }
    }

    pub fn last(&self) -> Option<ParamRef<'_>> {
        if let Some(ref p) = self.trailing {
            Some(ParamRef::Trailing(p))
        } else {
            self.middle.last().map(ParamRef::Middle)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = ParamRef<'_>> + '_ {
        self.middle
            .iter()
            .map(ParamRef::Middle)
            .chain(self.trailing.as_ref().map(ParamRef::Trailing))
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

impl fmt::Display for ParameterList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for p in self.iter() {
            if !std::mem::replace(&mut first, false) {
                write!(f, " ")?;
            }
            if p.is_middle() {
                write!(f, "{p}")?;
            } else {
                write!(f, ":{p}")?;
            }
        }
        Ok(())
    }
}

impl std::str::FromStr for ParameterList {
    type Err = ParseParameterListError;

    fn from_str(mut s: &str) -> Result<ParameterList, ParseParameterListError> {
        let mut middle = Vec::new();
        let mut trailing = None;
        while !s.is_empty() {
            if let Some(trail) = s.strip_prefix(':') {
                trailing = Some(trail.parse::<TrailingParam>()?);
                s = "";
            } else {
                let (param, rest) = split_word(s);
                middle.push(param.parse::<MiddleParam>()?);
                s = rest;
            }
        }
        Ok(ParameterList { middle, trailing })
    }
}

impl TryFrom<String> for ParameterList {
    type Error = TryFromStringError<ParseParameterListError>;

    fn try_from(
        string: String,
    ) -> Result<ParameterList, TryFromStringError<ParseParameterListError>> {
        match string.parse() {
            Ok(src) => Ok(src),
            Err(inner) => Err(TryFromStringError { inner, string }),
        }
    }
}

impl TryFrom<ParameterList> for () {
    type Error = TryFromParameterListError;

    fn try_from(params: ParameterList) -> Result<(), TryFromParameterListError> {
        if params.is_empty() {
            Ok(())
        } else {
            Err(TryFromParameterListError::ExactSizeMismatch {
                required: 0,
                received: params.len(),
            })
        }
    }
}

impl TryFrom<ParameterList> for (TrailingParam,) {
    type Error = TryFromParameterListError;

    fn try_from(mut params: ParameterList) -> Result<(TrailingParam,), TryFromParameterListError> {
        if params.len() == 1 {
            let p = params
                .middle
                .pop()
                .map(TrailingParam::from)
                .or(params.trailing)
                .expect("There should be something to unwrap when len is 1");
            Ok((p,))
        } else {
            Err(TryFromParameterListError::ExactSizeMismatch {
                required: 1,
                received: params.len(),
            })
        }
    }
}

impl TryFrom<ParameterList> for (Option<TrailingParam>,) {
    type Error = TryFromParameterListError;

    fn try_from(
        params: ParameterList,
    ) -> Result<(Option<TrailingParam>,), TryFromParameterListError> {
        match (params.middle.len(), params.trailing.is_some()) {
            (1, false) => Ok((params.middle.into_iter().next().map(TrailingParam::from),)),
            (0, _) => Ok((params.trailing,)),
            _ => Err(TryFromParameterListError::RangeSizeMismatch {
                min_required: 0,
                max_required: 1,
                received: params.len(),
            }),
        }
    }
}

impl TryFrom<ParameterList> for (MiddleParam, TrailingParam) {
    type Error = TryFromParameterListError;

    fn try_from(
        params: ParameterList,
    ) -> Result<(MiddleParam, TrailingParam), TryFromParameterListError> {
        if params.len() == 2 {
            let mut middles = params.middle.into_iter();
            let p1 = middles
                .next()
                .expect("First element should exist when len is 2");
            let p2 = middles
                .next()
                .map(TrailingParam::from)
                .or(params.trailing)
                .expect("Second element should exist when len is 2");
            Ok((p1, p2))
        } else {
            Err(TryFromParameterListError::ExactSizeMismatch {
                required: 2,
                received: params.len(),
            })
        }
    }
}

impl TryFrom<ParameterList> for (MiddleParam, Option<TrailingParam>) {
    type Error = TryFromParameterListError;

    fn try_from(
        params: ParameterList,
    ) -> Result<(MiddleParam, Option<TrailingParam>), TryFromParameterListError> {
        match (params.middle.len(), params.trailing.is_some()) {
            (2, false) => {
                let mut middles = params.middle.into_iter();
                let p1 = middles
                    .next()
                    .expect("First element should exist when len is 2");
                let p2 = middles.next().map(TrailingParam::from);
                Ok((p1, p2))
            }
            (1, _) => {
                let mut middles = params.middle.into_iter();
                let p1 = middles
                    .next()
                    .expect("First element should exist when len is 1");
                let p2 = params.trailing;
                Ok((p1, p2))
            }
            (0, true) => {
                let trailing = params.trailing.expect("trailing should be Some");
                match String::from(trailing).parse::<MiddleParam>() {
                    Ok(p1) => Ok((p1, None)),
                    Err(_) => Err(TryFromParameterListError::TrailingNotMiddle),
                }
            }
            _ => Err(TryFromParameterListError::RangeSizeMismatch {
                min_required: 1,
                max_required: 2,
                received: params.len(),
            }),
        }
    }
}

impl TryFrom<ParameterList> for (MiddleParam, MiddleParam, Option<TrailingParam>) {
    type Error = TryFromParameterListError;

    fn try_from(
        params: ParameterList,
    ) -> Result<(MiddleParam, MiddleParam, Option<TrailingParam>), TryFromParameterListError> {
        match (params.middle.len(), params.trailing.is_some()) {
            (3, false) => {
                let mut middles = params.middle.into_iter();
                let p1 = middles
                    .next()
                    .expect("First element should exist when len is 3");
                let p2 = middles
                    .next()
                    .expect("Second element should exist when len is 3");
                let p3 = middles.next().map(TrailingParam::from);
                Ok((p1, p2, p3))
            }
            (2, _) => {
                let mut middles = params.middle.into_iter();
                let p1 = middles
                    .next()
                    .expect("First element should exist when len is 2");
                let p2 = middles
                    .next()
                    .expect("Second element should exist when len is 2");
                let p3 = params.trailing;
                Ok((p1, p2, p3))
            }
            _ => Err(TryFromParameterListError::RangeSizeMismatch {
                min_required: 2,
                max_required: 3,
                received: params.len(),
            }),
        }
    }
}

impl TryFrom<ParameterList> for (MiddleParam, MiddleParam, TrailingParam) {
    type Error = TryFromParameterListError;

    fn try_from(
        params: ParameterList,
    ) -> Result<(MiddleParam, MiddleParam, TrailingParam), TryFromParameterListError> {
        if params.len() == 3 {
            let mut middles = params.middle.into_iter();
            let p1 = middles
                .next()
                .expect("First element should exist when len is 3");
            let p2 = middles
                .next()
                .expect("Second element should exist when len is 3");
            let p3 = middles
                .next()
                .map(TrailingParam::from)
                .or(params.trailing)
                .expect("Third element should exist when len is 3");
            Ok((p1, p2, p3))
        } else {
            Err(TryFromParameterListError::ExactSizeMismatch {
                required: 3,
                received: params.len(),
            })
        }
    }
}

impl TryFrom<ParameterList> for (MiddleParam, MiddleParam, MiddleParam, TrailingParam) {
    type Error = TryFromParameterListError;

    fn try_from(
        params: ParameterList,
    ) -> Result<(MiddleParam, MiddleParam, MiddleParam, TrailingParam), TryFromParameterListError>
    {
        if params.len() == 4 {
            let mut middles = params.middle.into_iter();
            let p1 = middles
                .next()
                .expect("First element should exist when len is 4");
            let p2 = middles
                .next()
                .expect("Second element should exist when len is 4");
            let p3 = middles
                .next()
                .expect("Third element should exist when len is 4");
            let p4 = middles
                .next()
                .map(TrailingParam::from)
                .or(params.trailing)
                .expect("Fourth element should exist when len is 4");
            Ok((p1, p2, p3, p4))
        } else {
            Err(TryFromParameterListError::ExactSizeMismatch {
                required: 4,
                received: params.len(),
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum ParseParameterListError {
    #[error(transparent)]
    Middle(#[from] ParseMiddleParamError),
    #[error(transparent)]
    Trailing(#[from] ParseTrailingParamError),
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum TryFromParameterListError {
    #[error("invalid number of parameters: {required} required, {received} received")]
    ExactSizeMismatch { required: usize, received: usize },
    #[error(
        "invalid number of parameters: {min_required}-{max_required} required, {received} received"
    )]
    RangeSizeMismatch {
        min_required: usize,
        max_required: usize,
        received: usize,
    },
    #[error("trailing parameter could not be converted to valid middle parameter")]
    TrailingNotMiddle,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct ParameterListBuilder(ParameterList);

impl ParameterListBuilder {
    pub fn new() -> ParameterListBuilder {
        ParameterListBuilder::default()
    }

    pub fn push_middle<P: Into<MiddleParam>>(&mut self, param: P) {
        self.0.middle.push(param.into());
    }

    pub fn with_middle<P: Into<MiddleParam>>(mut self, param: P) -> Self {
        self.push_middle(param);
        self
    }

    pub fn maybe_with_trailing<P: Into<TrailingParam>>(
        mut self,
        param: Option<P>,
    ) -> ParameterList {
        self.0.trailing = param.map(Into::into);
        self.0
    }

    pub fn with_trailing<P: Into<TrailingParam>>(mut self, param: P) -> ParameterList {
        self.0.trailing = Some(param.into());
        self.0
    }

    pub fn with_list(mut self, params: ParameterList) -> ParameterList {
        self.0.middle.extend(params.middle);
        self.0.trailing = params.trailing;
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
            .middle
            .into_iter()
            .map(Parameter::Middle)
            .collect::<Vec<_>>();
        paramvec.extend(params.trailing.map(Parameter::Trailing));
        ParameterListIntoIter(paramvec.into_iter())
    }

    #[expect(clippy::debug_assert_with_mut_call)]
    pub fn into_parameter_list(mut self) -> ParameterList {
        let mut builder = ParameterList::builder();
        for p in self.by_ref() {
            match p {
                Parameter::Middle(p) => builder.push_middle(p),
                Parameter::Trailing(p) => {
                    let params = builder.with_trailing(p);
                    debug_assert!(
                        self.next().is_none(),
                        "ParamterListIntoIter should be done after yielding a Trailing"
                    );
                    return params;
                }
            }
        }
        builder.finish()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trailing_into_one_or_two() {
        let params =
            ParameterList::builder().with_trailing("#testnet".parse::<TrailingParam>().unwrap());
        let (p1, p2): (_, Option<TrailingParam>) = params.try_into().unwrap();
        assert_eq!(p1, "#testnet");
        assert!(p2.is_none());
    }
}
