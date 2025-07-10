// <https://modern.ircdocs.horse> doesn't address the format of `<servername>`
// and `<host>` in source prefixes.
//
// RFC 1459 and RFC 2812 don't explicitly define the `<servername>` and
// `<host>` in `<prefix>`, but a few paragraphs after the definition of
// `<prefix>`, they both give BNF for targets in which `<servername>` and
// `<host>` are specified to be domain names.
//
// Based on <https://github.com/ircdocs/modern-irc/issues/168>, no validation
// should be performed on host segments — for now.

use crate::TryFromStringError;
use crate::types::{Nickname, ParseNicknameError, ParseUsernameError, Username};
use std::fmt;
use thiserror::Error;
use url::Host;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Source {
    Server(Host),
    Client(ClientSource),
}

impl Source {
    pub fn is_server(&self) -> bool {
        matches!(self, Source::Server(_))
    }

    pub fn is_client(&self) -> bool {
        matches!(self, Source::Client(_))
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::Server(server) => write!(f, "{server}"),
            Source::Client(client) => write!(f, "{client}"),
        }
    }
}

impl std::str::FromStr for Source {
    type Err = ParseSourceError;

    fn from_str(s: &str) -> Result<Source, ParseSourceError> {
        // cf. <https://github.com/ircdocs/modern-irc/issues/227>
        if !s.contains(['!', '@']) && s.contains('.') {
            Ok(Source::Server(Host::parse(s)?))
        } else {
            Ok(Source::Client(s.parse::<ClientSource>()?))
        }
    }
}

impl TryFrom<String> for Source {
    type Error = TryFromStringError<ParseSourceError>;

    fn try_from(string: String) -> Result<Source, TryFromStringError<ParseSourceError>> {
        match string.parse() {
            Ok(src) => Ok(src),
            Err(inner) => Err(TryFromStringError { inner, string }),
        }
    }
}

impl From<Host> for Source {
    fn from(value: Host) -> Source {
        Source::Server(value)
    }
}

impl From<ClientSource> for Source {
    fn from(value: ClientSource) -> Source {
        Source::Client(value)
    }
}

impl PartialEq<Host> for Source {
    fn eq(&self, other: &Host) -> bool {
        matches!(self, Source::Server(host) if host == other)
    }
}

impl PartialEq<ClientSource> for Source {
    fn eq(&self, other: &ClientSource) -> bool {
        matches!(self, Source::Client(client) if client == other)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientSource {
    pub nickname: Nickname,
    // Note that the user component may begin with a tilde if the IRC server
    // failed to look up the username using ident and is instead reporting a
    // username supplied with `USER`.
    pub user: Option<Username>,
    pub host: Option<String>,
}

impl fmt::Display for ClientSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.nickname)?;
        if let Some(ref user) = self.user {
            write!(f, "!{user}")?;
        }
        if let Some(ref host) = self.host {
            write!(f, "@{host}")?;
        }
        Ok(())
    }
}

impl std::str::FromStr for ClientSource {
    type Err = ParseClientSourceError;

    fn from_str(mut s: &str) -> Result<ClientSource, ParseClientSourceError> {
        let host_str = s.rsplit_once('@').map(|(pre, h)| {
            s = pre;
            h
        });
        let user_str = s.rsplit_once('!').map(|(pre, u)| {
            s = pre;
            u
        });
        let nickname = s.parse::<Nickname>()?;
        let user = user_str.map(str::parse::<Username>).transpose()?;
        let host = host_str.map(String::from);
        Ok(ClientSource {
            nickname,
            user,
            host,
        })
    }
}

impl TryFrom<String> for ClientSource {
    type Error = TryFromStringError<ParseClientSourceError>;

    fn try_from(
        string: String,
    ) -> Result<ClientSource, TryFromStringError<ParseClientSourceError>> {
        match string.parse() {
            Ok(src) => Ok(src),
            Err(inner) => Err(TryFromStringError { inner, string }),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseSourceError {
    #[error("invalid host")]
    Host(#[from] url::ParseError),
    #[error(transparent)]
    Client(#[from] ParseClientSourceError),
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseClientSourceError {
    #[error("invalid nickname")]
    Nickname(#[from] ParseNicknameError),
    #[error("invalid username")]
    Username(#[from] ParseUsernameError),
}

#[cfg(test)]
mod parser_tests {
    // Test cases from <https://github.com/ircdocs/parser-tests/blob/6b417e666de20ba677b14e0189213b3706009df6/tests/userhost-split.yaml>
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn simpler() {
        let source = "coolguy".parse::<Source>().unwrap();
        assert_matches!(source, Source::Client(ClientSource {
            nickname,
            user: None,
            host: None
        }) => {
            assert_eq!(nickname, "coolguy");
        });
    }

    #[test]
    fn simple1() {
        let source = "coolguy!ag@127.0.0.1".parse::<Source>().unwrap();
        assert_matches!(source, Source::Client(ClientSource {
            nickname,
            user: Some(user),
            host: Some(host)
        }) => {
            assert_eq!(nickname, "coolguy");
            assert_eq!(user, "ag");
            assert_eq!(host, "127.0.0.1");
        });
    }

    #[test]
    fn simple2() {
        let source = "coolguy!~ag@localhost".parse::<Source>().unwrap();
        assert_matches!(source, Source::Client(ClientSource {
            nickname,
            user: Some(user),
            host: Some(host)
        }) => {
            assert_eq!(nickname, "coolguy");
            assert_eq!(user, "~ag");
            assert_eq!(host, "localhost");
        });
    }

    #[test]
    fn without_user() {
        let source = "coolguy@127.0.0.1".parse::<Source>().unwrap();
        assert_matches!(source, Source::Client(ClientSource {
            nickname,
            user: None,
            host: Some(host)
        }) => {
            assert_eq!(nickname, "coolguy");
            assert_eq!(host, "127.0.0.1");
        });
    }

    #[test]
    fn without_host() {
        let source = "coolguy!ag".parse::<Source>().unwrap();
        assert_matches!(source, Source::Client(ClientSource {
            nickname,
            user: Some(user),
            host: None,
        }) => {
            assert_eq!(nickname, "coolguy");
            assert_eq!(user, "ag");
        });
    }

    #[test]
    fn control_codes1() {
        let source = "coolguy!ag@net\x035w\x03ork.admin"
            .parse::<Source>()
            .unwrap();
        assert_matches!(source, Source::Client(ClientSource {
            nickname,
            user: Some(user),
            host: Some(host)
        }) => {
            assert_eq!(nickname, "coolguy");
            assert_eq!(user, "ag");
            assert_eq!(host, "net\x035w\x03ork.admin");
        });
    }

    #[test]
    fn control_codes2() {
        let source = "coolguy!~ag@n\x02et\x0305w\x0fork.admin"
            .parse::<Source>()
            .unwrap();
        assert_matches!(source, Source::Client(ClientSource {
            nickname,
            user: Some(user),
            host: Some(host)
        }) => {
            assert_eq!(nickname, "coolguy");
            assert_eq!(user, "~ag");
            assert_eq!(host, "n\x02et\x0305w\x0fork.admin");
        });
    }
}
