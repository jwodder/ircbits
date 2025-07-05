use crate::linestream::IrcLineStream;
use irctext::{Nickname, Parameter, Username};

pub const PLAIN_PORT: u16 = 6667;

pub const TLS_PORT: u16 = 6697;

// Both RFC 2812 and <https://modern.ircdocs.horse> say that IRC messages (when
// tags aren't involved) are limited to 512 characters, counting the CR LF.
pub const MAX_LINE_LENGTH: usize = 512;

#[derive(Debug)]
pub struct Client {
    stream: IrcLineStream,
}

impl Client {
    pub fn new(stream: IrcLineStream) -> Client {
        Client { stream }
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    pub async fn register(&mut self, _reg: Registration) -> anyhow::Result<()> {
        todo!()
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    pub async fn list(&mut self) -> anyhow::Result<Vec<Channel>> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Registration {
    password: Option<Parameter>,
    nickname: Nickname,
    username: Username,
    realname: Parameter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Channel {
    name: String,
    clients: u32,
    topic: String,
}
