#![expect(unused_variables)]
use crate::{ParameterList, ToIrcLine};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Reply {
    Welcome(Welcome),
    YourHost(YourHost),
    Created(Created),
    MyInfo(MyInfo),
    ISupport(ISupport),
    // TODO
}

impl Reply {
    pub fn from_parts(code: u16, params: ParameterList) -> Result<Reply, ReplyError> {
        todo!()
    }
}

impl ToIrcLine for Reply {
    fn to_irc_line(&self) -> String {
        todo!()
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("TODO")]
pub struct ReplyError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Welcome;

impl TryFrom<ParameterList> for Welcome {
    type Error = ReplyError;

    fn try_from(params: ParameterList) -> Result<Welcome, ReplyError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YourHost;

impl TryFrom<ParameterList> for YourHost {
    type Error = ReplyError;

    fn try_from(params: ParameterList) -> Result<YourHost, ReplyError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Created;

impl TryFrom<ParameterList> for Created {
    type Error = ReplyError;

    fn try_from(params: ParameterList) -> Result<Created, ReplyError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MyInfo;

impl TryFrom<ParameterList> for MyInfo {
    type Error = ReplyError;

    fn try_from(params: ParameterList) -> Result<MyInfo, ReplyError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ISupport;

impl TryFrom<ParameterList> for ISupport {
    type Error = ReplyError;

    fn try_from(params: ParameterList) -> Result<ISupport, ReplyError> {
        todo!()
    }
}
