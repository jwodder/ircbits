/// Parsing of CTCP messages, as specified at
/// <https://datatracker.ietf.org/doc/html/draft-oakley-irc-ctcp-02>
use super::FinalParam;
use std::borrow::Cow;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CtcpMessage {
    Action(Option<CtcpParams>),
    ClientInfo(Option<CtcpParams>),
    Dcc(Option<CtcpParams>),
    Finger(Option<CtcpParams>),
    Ping(Option<CtcpParams>),
    Source(Option<CtcpParams>),
    Time(Option<CtcpParams>),
    UserInfo(Option<CtcpParams>),
    Version(Option<CtcpParams>),
    Other {
        command: CtcpCommand,
        params: Option<CtcpParams>,
    },
    /// Not a valid CTCP message, just normal text
    Plain(FinalParam),
}

impl CtcpMessage {
    pub fn is_action(&self) -> bool {
        matches!(self, CtcpMessage::Action(_))
    }

    pub fn is_clientinfo(&self) -> bool {
        matches!(self, CtcpMessage::ClientInfo(_))
    }

    pub fn is_dcc(&self) -> bool {
        matches!(self, CtcpMessage::Dcc(_))
    }

    pub fn is_finger(&self) -> bool {
        matches!(self, CtcpMessage::Finger(_))
    }

    pub fn is_ping(&self) -> bool {
        matches!(self, CtcpMessage::Ping(_))
    }

    pub fn is_source(&self) -> bool {
        matches!(self, CtcpMessage::Source(_))
    }

    pub fn is_time(&self) -> bool {
        matches!(self, CtcpMessage::Time(_))
    }

    pub fn is_userinfo(&self) -> bool {
        matches!(self, CtcpMessage::UserInfo(_))
    }

    pub fn is_version(&self) -> bool {
        matches!(self, CtcpMessage::Version(_))
    }

    pub fn is_other(&self) -> bool {
        matches!(self, CtcpMessage::Other { .. })
    }

    pub fn is_plain(&self) -> bool {
        matches!(self, CtcpMessage::Plain(_))
    }
}

impl From<FinalParam> for CtcpMessage {
    fn from(p: FinalParam) -> CtcpMessage {
        let Some(txt) = p.as_str().strip_prefix('\x01') else {
            return CtcpMessage::Plain(p);
        };
        let txt = txt.strip_suffix('\x01').unwrap_or(txt);
        let (cmd, params) = txt.split_once(' ').unwrap_or((txt, ""));
        let Ok(cmd) = cmd.parse::<CtcpCommand>() else {
            return CtcpMessage::Plain(p);
        };
        let params = if params.is_empty() {
            None
        } else if let Ok(ps) = params.parse::<CtcpParams>() {
            Some(ps)
        } else {
            return CtcpMessage::Plain(p);
        };
        if cmd.as_str().eq_ignore_ascii_case("ACTION") {
            CtcpMessage::Action(params)
        } else if cmd.as_str().eq_ignore_ascii_case("CLIENTINFO") {
            CtcpMessage::ClientInfo(params)
        } else if cmd.as_str().eq_ignore_ascii_case("DCC") {
            CtcpMessage::Dcc(params)
        } else if cmd.as_str().eq_ignore_ascii_case("FINGER") {
            CtcpMessage::Finger(params)
        } else if cmd.as_str().eq_ignore_ascii_case("PING") {
            CtcpMessage::Ping(params)
        } else if cmd.as_str().eq_ignore_ascii_case("SOURCE") {
            CtcpMessage::Source(params)
        } else if cmd.as_str().eq_ignore_ascii_case("TIME") {
            CtcpMessage::Time(params)
        } else if cmd.as_str().eq_ignore_ascii_case("USERINFO") {
            CtcpMessage::UserInfo(params)
        } else if cmd.as_str().eq_ignore_ascii_case("VERSION") {
            CtcpMessage::Version(params)
        } else {
            CtcpMessage::Other {
                command: cmd,
                params,
            }
        }
    }
}

impl From<CtcpMessage> for FinalParam {
    fn from(msg: CtcpMessage) -> FinalParam {
        let (cmd, params) = match msg {
            CtcpMessage::Action(params) => (Cow::from("ACTION"), params),
            CtcpMessage::ClientInfo(params) => (Cow::from("CLIENTINFO"), params),
            CtcpMessage::Dcc(params) => (Cow::from("DCC"), params),
            CtcpMessage::Finger(params) => (Cow::from("FINGER"), params),
            CtcpMessage::Ping(params) => (Cow::from("PING"), params),
            CtcpMessage::Source(params) => (Cow::from("SOURCE"), params),
            CtcpMessage::Time(params) => (Cow::from("TIME"), params),
            CtcpMessage::UserInfo(params) => (Cow::from("USERINFO"), params),
            CtcpMessage::Version(params) => (Cow::from("VERSION"), params),
            CtcpMessage::Other { command, params } => (Cow::from(command.to_string()), params),
            CtcpMessage::Plain(fp) => return fp,
        };
        let s = if let Some(ps) = params {
            format!("\x01{cmd} {ps}\x01")
        } else {
            format!("\x01{cmd}\x01")
        };
        FinalParam::try_from(s).expect("Formatted CTCP message should be valid FinalParam")
    }
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CtcpCommand(String);

validstr!(CtcpCommand, ParseCtcpCommandError, validate_cmd);

impl From<CtcpCommand> for FinalParam {
    fn from(value: CtcpCommand) -> FinalParam {
        FinalParam::try_from(value.into_inner()).expect("CTCP commands should be valid FinalParam")
    }
}

fn validate_cmd(s: &str) -> Result<(), ParseCtcpCommandError> {
    if s.is_empty() {
        Err(ParseCtcpCommandError::Empty)
    } else if s.contains(['\0', '\x01', '\r', '\n', ' ']) {
        Err(ParseCtcpCommandError::BadCharacter)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum ParseCtcpCommandError {
    #[error("CTCP commands cannot be empty")]
    Empty,
    #[error("CTCP commands cannot contain NUL, Ctrl-A, CR, LF, or SPACE")]
    BadCharacter,
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CtcpParams(String);

validstr!(CtcpParams, ParseCtcpParamsError, validate_params);

impl From<CtcpParams> for FinalParam {
    fn from(value: CtcpParams) -> FinalParam {
        FinalParam::try_from(value.into_inner()).expect("CTCP params should be valid FinalParam")
    }
}

fn validate_params(s: &str) -> Result<(), ParseCtcpParamsError> {
    if s.is_empty() {
        Err(ParseCtcpParamsError::Empty)
    } else if s.contains(['\0', '\x01', '\r', '\n']) {
        Err(ParseCtcpParamsError::BadCharacter)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum ParseCtcpParamsError {
    #[error("CTCP parameters cannot be empty")]
    Empty,
    #[error("CTCP parameters cannot contain NUL, Ctrl-A, CR, or LF")]
    BadCharacter,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use rstest::rstest;

    #[test]
    fn version_no_params() {
        let p = "\x01VERSION\x01".parse::<FinalParam>().unwrap();
        let ctcp = CtcpMessage::from(p);
        assert_eq!(ctcp, CtcpMessage::Version(None));
    }

    #[test]
    fn version_params() {
        let p = "\x01VERSION Snak for Mac 4.13\x01"
            .parse::<FinalParam>()
            .unwrap();
        let ctcp = CtcpMessage::from(p);
        assert_matches!(ctcp, CtcpMessage::Version(Some(ps)) => {
            assert_eq!(ps, "Snak for Mac 4.13");
        });
    }

    #[test]
    fn ping() {
        let p = "\x01PING 1473523796 918320\x01"
            .parse::<FinalParam>()
            .unwrap();
        let ctcp = CtcpMessage::from(p);
        assert_matches!(ctcp, CtcpMessage::Ping(Some(ps)) => {
            assert_eq!(ps, "1473523796 918320");
        });
    }

    #[test]
    fn action() {
        let p = "\x01ACTION writes some specs!\x01"
            .parse::<FinalParam>()
            .unwrap();
        let ctcp = CtcpMessage::from(p);
        assert_matches!(ctcp, CtcpMessage::Action(Some(ps)) => {
            assert_eq!(ps, "writes some specs!");
        });
    }

    #[rstest]
    #[case("\x01ACTION \x01")]
    #[case("\x01ACTION\x01")]
    #[case("\x01ACTION")]
    fn action_no_param(#[case] s: &str) {
        let p = s.parse::<FinalParam>().unwrap();
        let ctcp = CtcpMessage::from(p);
        assert_eq!(ctcp, CtcpMessage::Action(None));
    }
}
