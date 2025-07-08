use crate::util::split_word;
use crate::{
    Command, ParameterList, ParseCommandError, ParseParameterListError, ParseSourceError, Source,
    TryFromStringError,
};
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawMessage {
    pub source: Option<Source>,
    pub command: Command,
    pub parameters: ParameterList,
}

impl fmt::Display for RawMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(source) = self.source.as_ref() {
            write!(f, ":{source} ")?;
        }
        write!(f, "{}", self.command)?;
        for p in self.parameters.iter() {
            if p.is_medial() {
                write!(f, " {p}")?;
            } else {
                write!(f, " :{p}")?;
            }
        }
        Ok(())
    }
}

impl std::str::FromStr for RawMessage {
    type Err = ParseRawMessageError;

    // `s` may optionally end with LF, CR LF, or CR.
    fn from_str(s: &str) -> Result<RawMessage, ParseRawMessageError> {
        let mut s = s.strip_suffix('\n').unwrap_or(s);
        s = s.strip_suffix('\r').unwrap_or(s);
        let source = if let Some(s2) = s.strip_prefix(':') {
            let (source_str, rest) = split_word(s2);
            s = rest;
            Some(source_str.parse::<Source>()?)
        } else {
            None
        };
        let (cmd_str, params) = split_word(s);
        let command = cmd_str.parse::<Command>()?;
        let parameters = params.parse::<ParameterList>()?;
        Ok(RawMessage {
            source,
            command,
            parameters,
        })
    }
}

impl TryFrom<String> for RawMessage {
    type Error = TryFromStringError<ParseRawMessageError>;

    fn try_from(string: String) -> Result<RawMessage, TryFromStringError<ParseRawMessageError>> {
        match string.parse() {
            Ok(msg) => Ok(msg),
            Err(inner) => Err(TryFromStringError { inner, string }),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseRawMessageError {
    #[error("invalid source prefix")]
    Source(#[from] ParseSourceError),
    #[error("invalid command")]
    Command(#[from] ParseCommandError),
    #[error("invalid parameter")]
    Parameter(#[from] ParseParameterListError),
}

#[cfg(test)]
mod parser_tests {
    // Test cases from <https://github.com/ircdocs/parser-tests/blob/6b417e666de20ba677b14e0189213b3706009df6/tests/msg-split.yaml>
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn simple() {
        let msg = "foo bar baz asdf".parse::<RawMessage>().unwrap();
        assert!(msg.source.is_none());
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "foo");
        });
        assert_eq!(msg.parameters, ["bar", "baz", "asdf"]);
    }

    #[test]
    fn with_source() {
        let msg = ":coolguy foo bar baz asdf".parse::<RawMessage>().unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "coolguy");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "foo");
        });
        assert_eq!(msg.parameters, ["bar", "baz", "asdf"]);
    }

    #[test]
    fn with_trailing_param1() {
        let msg = "foo bar baz :asdf quux".parse::<RawMessage>().unwrap();
        assert!(msg.source.is_none());
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "foo");
        });
        assert_eq!(msg.parameters, ["bar", "baz", "asdf quux"]);
    }

    #[test]
    fn with_trailing_param2() {
        let msg = "foo bar baz :".parse::<RawMessage>().unwrap();
        assert!(msg.source.is_none());
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "foo");
        });
        assert_eq!(msg.parameters, ["bar", "baz", ""]);
    }

    #[test]
    fn with_trailing_param3() {
        let msg = "foo bar baz ::asdf".parse::<RawMessage>().unwrap();
        assert!(msg.source.is_none());
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "foo");
        });
        assert_eq!(msg.parameters, ["bar", "baz", ":asdf"]);
    }

    #[test]
    fn with_source_and_trailing_param1() {
        let msg = ":coolguy foo bar baz :asdf quux"
            .parse::<RawMessage>()
            .unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "coolguy");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "foo");
        });
        assert_eq!(msg.parameters, ["bar", "baz", "asdf quux"]);
    }

    #[test]
    fn with_source_and_trailing_param2() {
        let msg = ":coolguy foo bar baz :  asdf quux "
            .parse::<RawMessage>()
            .unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "coolguy");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "foo");
        });
        assert_eq!(msg.parameters, ["bar", "baz", "  asdf quux "]);
    }

    #[test]
    fn with_source_and_trailing_param3() {
        let msg = ":coolguy PRIVMSG bar :lol :) "
            .parse::<RawMessage>()
            .unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "coolguy");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "PRIVMSG");
        });
        assert_eq!(msg.parameters, ["bar", "lol :) "]);
    }

    #[test]
    fn with_source_and_trailing_param4() {
        let msg = ":coolguy foo bar baz :".parse::<RawMessage>().unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "coolguy");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "foo");
        });
        assert_eq!(msg.parameters, ["bar", "baz", ""]);
    }

    #[test]
    fn with_source_and_trailing_param5() {
        let msg = ":coolguy foo bar baz :  ".parse::<RawMessage>().unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "coolguy");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "foo");
        });
        assert_eq!(msg.parameters, ["bar", "baz", "  "]);
    }

    #[test]
    fn last_param1() {
        let msg = ":src JOIN #chan".parse::<RawMessage>().unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "src");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "JOIN");
        });
        assert_eq!(msg.parameters, ["#chan"]);
    }

    #[test]
    fn last_param2() {
        let msg = ":src JOIN :#chan".parse::<RawMessage>().unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "src");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "JOIN");
        });
        assert_eq!(msg.parameters, ["#chan"]);
    }

    #[test]
    fn without_last_param() {
        let msg = ":src AWAY".parse::<RawMessage>().unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "src");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "AWAY");
        });
        assert!(msg.parameters.is_empty());
    }

    #[test]
    fn with_last_param() {
        let msg = ":src AWAY ".parse::<RawMessage>().unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "src");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "AWAY");
        });
        assert!(msg.parameters.is_empty());
    }

    #[test]
    fn tab_not_space() {
        let msg = ":cool\tguy foo bar baz".parse::<RawMessage>().unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "cool\tguy");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "foo");
        });
        assert_eq!(msg.parameters, ["bar", "baz"]);
    }

    #[test]
    fn control_code_source1() {
        let msg = ":coolguy!ag@net\x035w\x03ork.admin PRIVMSG foo :bar baz"
            .parse::<RawMessage>()
            .unwrap();
        assert_eq!(
            msg.source.unwrap().to_string(),
            "coolguy!ag@net\x035w\x03ork.admin"
        );
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "PRIVMSG");
        });
        assert_eq!(msg.parameters, ["foo", "bar baz"]);
    }

    #[test]
    fn control_code_source2() {
        let msg = ":coolguy!~ag@n\x02et\x0305w\x0fork.admin PRIVMSG foo :bar baz"
            .parse::<RawMessage>()
            .unwrap();
        assert_eq!(
            msg.source.unwrap().to_string(),
            "coolguy!~ag@n\x02et\x0305w\x0fork.admin"
        );
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "PRIVMSG");
        });
        assert_eq!(msg.parameters, ["foo", "bar baz"]);
    }

    #[test]
    fn misc01() {
        let msg = ":irc.example.com COMMAND param1 param2 :param3 param3"
            .parse::<RawMessage>()
            .unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "irc.example.com");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "COMMAND");
        });
        assert_eq!(msg.parameters, ["param1", "param2", "param3 param3"]);
    }

    #[test]
    fn just_command() {
        let msg = "COMMAND".parse::<RawMessage>().unwrap();
        assert!(msg.source.is_none());
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "COMMAND");
        });
        assert!(msg.parameters.is_empty());
    }

    #[test]
    fn unreal01() {
        let msg = ":gravel.mozilla.org 432  #momo :Erroneous Nickname: Illegal characters"
            .parse::<RawMessage>()
            .unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "gravel.mozilla.org");
        assert_eq!(msg.command, Command::Reply(432));
        assert_eq!(
            msg.parameters,
            ["#momo", "Erroneous Nickname: Illegal characters"]
        );
    }

    #[test]
    fn unreal02() {
        let msg = ":gravel.mozilla.org MODE #tckk +n "
            .parse::<RawMessage>()
            .unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "gravel.mozilla.org");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "MODE");
        });
        assert_eq!(msg.parameters, ["#tckk", "+n"]);
    }

    #[test]
    fn unreal03() {
        let msg = ":services.esper.net MODE #foo-bar +o foobar  "
            .parse::<RawMessage>()
            .unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "services.esper.net");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "MODE");
        });
        assert_eq!(msg.parameters, ["#foo-bar", "+o", "foobar"]);
    }

    #[test]
    fn mode01() {
        let msg = ":SomeOp MODE #channel :+i".parse::<RawMessage>().unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "SomeOp");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "MODE");
        });
        assert_eq!(msg.parameters, ["#channel", "+i"]);
    }

    #[test]
    fn mode02() {
        let msg = ":SomeOp MODE #channel +oo SomeUser :AnotherUser"
            .parse::<RawMessage>()
            .unwrap();
        assert_eq!(msg.source.unwrap().to_string(), "SomeOp");
        assert_matches!(msg.command, Command::Verb(v) => {
            assert_eq!(v, "MODE");
        });
        assert_eq!(
            msg.parameters,
            ["#channel", "+oo", "SomeUser", "AnotherUser"]
        );
    }
}
