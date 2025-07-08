use crate::types::{Nickname, ParseNicknameError, ParseUsernameError, Username};
use std::fmt;
use thiserror::Error;
use url::Host;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Source {
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
        nickname: Nickname,
        // Note that the user component may begin with a tilde if the IRC
        // server failed to look up the username using ident and is instead
        // reporting a username supplied with `USER`.  TODO: Extract the tilde
        // as a field?
        user: Option<Username>,
        host: Option<String>,
    },
}

impl fmt::Display for Source {
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

impl std::str::FromStr for Source {
    type Err = ParseSourceError;

    fn from_str(s: &str) -> Result<Source, ParseSourceError> {
        String::from(s).try_into()
    }
}

impl TryFrom<String> for Source {
    type Error = ParseSourceError;

    fn try_from(s: String) -> Result<Source, ParseSourceError> {
        // cf. <https://github.com/ircdocs/modern-irc/issues/227>
        if !s.contains(['!', '@']) && s.contains('.') {
            Ok(Source::Server(Host::parse(&s)?))
        } else {
            let mut s = &*s;
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
            Ok(Source::Client {
                nickname,
                user,
                host,
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseSourceError {
    #[error("invalid host")]
    Host(#[from] url::ParseError),
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
        assert_matches!(source, Source::Client {
            nickname,
            user: None,
            host: None
        } => {
            assert_eq!(nickname, "coolguy");
        });
    }

    #[test]
    fn simple1() {
        let source = "coolguy!ag@127.0.0.1".parse::<Source>().unwrap();
        assert_matches!(source, Source::Client {
            nickname,
            user: Some(user),
            host: Some(host)
        } => {
            assert_eq!(nickname, "coolguy");
            assert_eq!(user, "ag");
            assert_eq!(host, "127.0.0.1");
        });
    }

    #[test]
    fn simple2() {
        let source = "coolguy!~ag@localhost".parse::<Source>().unwrap();
        assert_matches!(source, Source::Client {
            nickname,
            user: Some(user),
            host: Some(host)
        } => {
            assert_eq!(nickname, "coolguy");
            assert_eq!(user, "~ag");
            assert_eq!(host, "localhost");
        });
    }

    #[test]
    fn without_user() {
        let source = "coolguy@127.0.0.1".parse::<Source>().unwrap();
        assert_matches!(source, Source::Client {
            nickname,
            user: None,
            host: Some(host)
        } => {
            assert_eq!(nickname, "coolguy");
            assert_eq!(host, "127.0.0.1");
        });
    }

    #[test]
    fn without_host() {
        let source = "coolguy!ag".parse::<Source>().unwrap();
        assert_matches!(source, Source::Client {
            nickname,
            user: Some(user),
            host: None,
        } => {
            assert_eq!(nickname, "coolguy");
            assert_eq!(user, "ag");
        });
    }

    #[test]
    fn control_codes1() {
        let source = "coolguy!ag@net\x035w\x03ork.admin"
            .parse::<Source>()
            .unwrap();
        assert_matches!(source, Source::Client {
            nickname,
            user: Some(user),
            host: Some(host)
        } => {
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
        assert_matches!(source, Source::Client {
            nickname,
            user: Some(user),
            host: Some(host)
        } => {
            assert_eq!(nickname, "coolguy");
            assert_eq!(user, "~ag");
            assert_eq!(host, "n\x02et\x0305w\x0fork.admin");
        });
    }
}
