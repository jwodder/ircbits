#![expect(unused_variables)]
use crate::{ParameterList, Verb};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClientMessage {
    Join(Join),
    Kick(Kick),
    Mode(Mode),
    Nick(Nick),
    Notice(Notice),
    Part(Part),
    Pass(Pass),
    Ping(Ping),
    Pong(Pong),
    PrivMsg(PrivMsg),
    Quit(Quit),
    User(User),
    // TODO
}

impl ClientMessage {
    pub fn from_parts(
        verb: Verb,
        params: ParameterList,
    ) -> Result<ClientMessage, ClientMessageError> {
        todo!()
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("TODO")]
pub struct ClientMessageError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Join;

impl TryFrom<ParameterList> for Join {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Join, ClientMessageError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Kick;

impl TryFrom<ParameterList> for Kick {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Kick, ClientMessageError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mode;

impl TryFrom<ParameterList> for Mode {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Mode, ClientMessageError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Nick;

impl TryFrom<ParameterList> for Nick {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Nick, ClientMessageError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Notice;

impl TryFrom<ParameterList> for Notice {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Notice, ClientMessageError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Part;

impl TryFrom<ParameterList> for Part {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Part, ClientMessageError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pass;

impl TryFrom<ParameterList> for Pass {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Pass, ClientMessageError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ping;

impl TryFrom<ParameterList> for Ping {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Ping, ClientMessageError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pong;

impl TryFrom<ParameterList> for Pong {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Pong, ClientMessageError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivMsg;

impl TryFrom<ParameterList> for PrivMsg {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<PrivMsg, ClientMessageError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Quit;

impl TryFrom<ParameterList> for Quit {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Quit, ClientMessageError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User;

impl TryFrom<ParameterList> for User {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<User, ClientMessageError> {
        todo!()
    }
}
