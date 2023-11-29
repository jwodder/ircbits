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

    pub(crate) async fn register(&mut self, reg: Registration<'_>) -> anyhow::Result<()> {
        todo!()
    }

    pub(crate) async fn list(&mut self) -> anyhow::Result<Vec<Channel>> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Registration<'a> {
    password: Option<Parameter<'a>>,
    nickname: Nickname<'a>,
    username: Username<'a>,
    realname: Parameter<'a>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Channel {
    name: String,
    clients: u32,
    topic: String,
}
