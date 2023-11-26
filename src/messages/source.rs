use super::nickname::{Nickname, NicknameError};
use super::username::{Username, UsernameError};
use std::borrow::Cow;
use std::fmt;
use thiserror::Error;
use url::Host;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Source<'a> {
    // <https://modern.ircdocs.horse> doesn't address the format of
    // `<servername>` and `<host>` in source prefixes.
    //
    // RFC 1459 and RFC 2812 don't explicitly define the `<servername>` and
    // `<host>` in `<prefix>`, but a few paragraphs after the definition of
    // `<prefix>`, they both give BNF for targets in which `<servername>` and
    // `<host>` are specified to be domain names.
    //
    // Based on <https://github.com/ircdocs/modern-irc/issues/168>, no
    // validation should be performed on host segments — for now.
    Server(Host),
    Client {
        nickname: Nickname<'a>,
        // Note that the user component may begin with a tilde if the IRC
        // server failed to look up the username using ident and is instead
        // reporting a username supplied with `USER`.  TODO: Extract the tilde
        // as a field?
        user: Option<Username<'a>>,
        host: Option<Cow<'a, str>>,
    },
}

impl fmt::Display for Source<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::Server(server) => write!(f, "{server}"),
            Source::Client {
                nickname,
                user,
                host,
            } => {
                write!(f, "{nickname}")?;
                if let Some(user) = user {
                    write!(f, "!{user}")?;
                }
                if let Some(host) = host {
                    write!(f, "@{host}")?;
                }
                Ok(())
            }
        }
    }
}

impl<'a> TryFrom<&'a str> for Source<'a> {
    type Error = SourceError;

    fn try_from(mut s: &'a str) -> Result<Source<'a>, SourceError> {
        // cf. <https://github.com/ircdocs/modern-irc/issues/227>
        if !s.contains(['!', '@']) && s.contains('.') {
            Ok(Source::Server(Host::parse(s)?))
        } else {
            let host_str = s.rsplit_once('@').map(|(pre, h)| {
                s = pre;
                h
            });
            let user_str = s.rsplit_once('!').map(|(pre, u)| {
                s = pre;
                u
            });
            let nickname = Nickname::try_from(s)?;
            let user = user_str.map(Username::try_from).transpose()?;
            let host = host_str.map(Into::into);
            Ok(Source::Client {
                nickname,
                user,
                host,
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum SourceError {
    #[error("invalid host")]
    Host(#[from] url::ParseError),
    #[error("invalid nickname")]
    Nickname(#[from] NicknameError),
    #[error("invalid username")]
    Username(#[from] UsernameError),
}
