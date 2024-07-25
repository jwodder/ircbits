use crate::linestream::IrcLineStream;
use crate::messages::{Nickname, Parameter, Username};

pub(crate) const PLAIN_PORT: u16 = 6667;

pub(crate) const TLS_PORT: u16 = 6697;

// Both RFC 2812 and <https://modern.ircdocs.horse> say that IRC messages (when
// tags aren't involved) are limited to 512 characters, counting the CR LF.
pub(crate) const MAX_LINE_LENGTH: usize = 512;

#[derive(Debug)]
pub(crate) struct Client {
    stream: IrcLineStream,
}

impl Client {
    pub(crate) fn new(stream: IrcLineStream) -> Client {
        Client { stream }
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    pub(crate) async fn register(&mut self, reg: Registration) -> anyhow::Result<()> {
        todo!()
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    pub(crate) async fn list(&mut self) -> anyhow::Result<Vec<Channel>> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Registration {
    password: Option<Parameter>,
    nickname: Nickname,
    username: Username,
    realname: Parameter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Channel {
    name: String,
    clients: u32,
    topic: String,
}
