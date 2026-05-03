use super::Command;
use irctext::{
    ClientMessage, Message, Payload, Reply, ReplyParts,
    clientmsgs::Mode,
    types::{ModeString, ModeTarget, MsgTarget},
};
use std::time::Duration;
use thiserror::Error;

/// How long to wait for an optional `ERR_UMODEUNKNOWNFLAG` (501) message after
/// receiving a `MODE` response or for a `MODE` response after receiving an
/// `ERR_UMODEUNKNOWNFLAG` response
const ERR501_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetUserMode {
    outgoing: Vec<ClientMessage>,
    state: State,
}

impl SetUserMode {
    pub fn new<T: Into<ModeTarget>>(target: T, modestring: ModeString) -> SetUserMode {
        SetUserMode {
            outgoing: vec![Mode::new_with_modestring(target.into(), modestring).into()],
            state: State::Start,
        }
    }
}

// Order of replies on sucessful MODE:
//  - MODE
//  - optional ERR_UMODEUNKNOWNFLAG (501)
// OR (theoretically):
//  - ERR_UMODEUNKNOWNFLAG (501)
//  - MODE

// Possible error replies:
//  - ERROR message
//  - ERR_NOSUCHNICK (401)
//  - ERR_USERSDONTMATCH (502)
//  - RPL_TRYAGAIN (263)
//  - ERR_INPUTTOOLONG (417)
//  - ERR_UNKNOWNCOMMAND (421)
//  - ERR_NOTREGISTERED (451)
//  - ERR_NEEDMOREPARAMS (461) ?
//  - ERR_UMODEUNKNOWNFLAG (501)

impl Command for SetUserMode {
    type Output = SetUserModeOutput;
    type Error = SetUserModeError;

    fn get_client_messages(&mut self) -> Vec<ClientMessage> {
        std::mem::take(&mut self.outgoing)
    }

    fn handle_message(&mut self, msg: &Message) -> bool {
        match &msg.payload {
            Payload::Reply(Reply::UmodeUnknownFlag(r)) => {
                match std::mem::replace(&mut self.state, State::Void) {
                    State::Start => {
                        self.state = State::GotUmodeUnknown {
                            message: r.message().to_owned(),
                            timeout: Some(ERR501_TIMEOUT),
                        };
                        true
                    }
                    State::GotNewMode { mode, .. } => {
                        self.state = State::Done(Some(Ok(SetUserModeOutput {
                            new_mode: mode,
                            umode_unknown_message: Some(r.message().to_owned()),
                        })));
                        true
                    }
                    st => {
                        self.state = st;
                        false
                    }
                }
            }
            Payload::Reply(rpl) if rpl.is_error() && !matches!(rpl, Reply::NoMotd(_)) => {
                if self.state != State::Start {
                    return false;
                }
                let e = match rpl {
                    Reply::NoSuchNick(r) => SetUserModeError::NoSuchNick {
                        nickname: r.target().to_owned(),
                        message: r.message().to_owned(),
                    },
                    Reply::UsersDontMatch(r) => SetUserModeError::UsersDontMatch {
                        message: r.message().to_owned(),
                    },
                    Reply::TryAgain(r) => SetUserModeError::TryAgain {
                        message: r.message().to_owned(),
                    },
                    Reply::InputTooLong(r) => SetUserModeError::InputTooLong {
                        message: r.message().to_string(),
                    },
                    Reply::UnknownCommand(r) => SetUserModeError::UnknownCommand {
                        command: r.command().to_string(),
                        message: r.message().to_string(),
                    },
                    Reply::NotRegistered(r) => SetUserModeError::NotRegistered {
                        message: r.message().to_string(),
                    },
                    unexpected => SetUserModeError::UnexpectedError {
                        code: unexpected.code(),
                        reply: msg.to_string(),
                    },
                };
                self.state = State::Done(Some(Err(e)));
                true
            }
            Payload::ClientMessage(ClientMessage::Error(err)) => {
                self.state = State::Done(Some(Err(SetUserModeError::ErrorMessage {
                    reason: err.reason().to_string(),
                })));
                true
            }
            Payload::ClientMessage(ClientMessage::Mode(m)) => match (&self.state, m.modestring()) {
                (State::Start, Some(ms)) => {
                    self.state = State::GotNewMode {
                        mode: ms.to_owned(),
                        timeout: Some(ERR501_TIMEOUT),
                    };
                    true
                }
                (State::GotUmodeUnknown { message, .. }, Some(ms)) => {
                    self.state = State::Done(Some(Ok(SetUserModeOutput {
                        new_mode: ms.to_owned(),
                        umode_unknown_message: Some(message.clone()),
                    })));
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn get_timeout(&mut self) -> Option<Duration> {
        match &mut self.state {
            State::GotNewMode { timeout, .. } => timeout.take(),
            State::GotUmodeUnknown { timeout, .. } => timeout.take(),
            _ => None,
        }
    }

    fn handle_timeout(&mut self) {
        let state = std::mem::replace(&mut self.state, State::Void);
        self.state = match state {
            State::GotNewMode {
                timeout: None,
                mode,
            } => State::Done(Some(Ok(SetUserModeOutput {
                new_mode: mode,
                umode_unknown_message: None,
            }))),
            State::GotUmodeUnknown {
                timeout: None,
                message,
            } => State::Done(Some(Err(SetUserModeError::UmodeUnknownFlag { message }))),
            other => other,
        };
    }

    fn is_done(&self) -> bool {
        matches!(self.state, State::Done(_))
    }

    fn get_output(&mut self) -> Result<SetUserModeOutput, SetUserModeError> {
        if let State::Done(ref mut r) = self.state {
            r.take()
                .expect("get_output() should not be called more than once")
        } else {
            panic!("get_output() should only be called when is_done() is true");
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum State {
    Start,
    GotNewMode {
        mode: ModeString,
        timeout: Option<Duration>,
    },
    GotUmodeUnknown {
        message: String,
        timeout: Option<Duration>,
    },
    Done(Option<Result<SetUserModeOutput, SetUserModeError>>),
    Void,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetUserModeOutput {
    new_mode: ModeString,
    umode_unknown_message: Option<String>,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SetUserModeError {
    #[error("Set user mode failed: no such nick {nickname:?}: {message:?}")]
    NoSuchNick {
        nickname: MsgTarget,
        message: String,
    },
    #[error("JOIN failed: users don't match: {message:?}")]
    UsersDontMatch { message: String },
    #[error("JOIN failed: unknown user mode flag: {message:?}")]
    UmodeUnknownFlag { message: String },
    #[error("JOIN failed: try again later: {message:?}")]
    TryAgain { message: String },
    #[error("JOIN failed: registration required: {message:?}")]
    NotRegistered { message: String },
    #[error("JOIN failed due to overly-long input line: {message:?}")]
    InputTooLong { message: String },
    #[error("JOIN failed because server does not recognize {command:?} command: {message:?}")]
    UnknownCommand { command: String, message: String },
    #[error("server sent ERROR message during JOIN: {reason:?}")]
    ErrorMessage { reason: String },
    #[error("JOIN failed with unexpected error reply {code:03}: {reply:?}")]
    UnexpectedError { code: u16, reply: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use irctext::ClientMessageParts;

    #[test]
    fn simple() {
        let target = "luser".parse::<ModeTarget>().unwrap();
        let mstr = "+q".parse::<ModeString>().unwrap();
        let mut cmd = SetUserMode::new(target, mstr);
        let outgoing = cmd
            .get_client_messages()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["MODE luser +q"]);
        let msg = ":irc.example.com MODE luser +Zqw"
            .parse::<Message>()
            .unwrap();
        assert!(cmd.handle_message(&msg));
        assert!(cmd.get_client_messages().is_empty());
        assert!(!cmd.is_done());
        assert!(cmd.get_timeout().is_some());
        cmd.handle_timeout();
        assert!(cmd.get_client_messages().is_empty());
        assert!(cmd.is_done());
        let output = cmd.get_output().unwrap();
        assert_eq!(
            output,
            SetUserModeOutput {
                new_mode: "+Zqw".parse::<ModeString>().unwrap(),
                umode_unknown_message: None
            }
        );
    }

    #[test]
    fn good_and_bad_modes_mode_reply_first() {
        let target = "luser".parse::<ModeTarget>().unwrap();
        let mstr = "+Qq".parse::<ModeString>().unwrap();
        let mut cmd = SetUserMode::new(target, mstr);
        let outgoing = cmd
            .get_client_messages()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["MODE luser +Qq"]);
        let msg = ":irc.example.com MODE luser +Zqw"
            .parse::<Message>()
            .unwrap();
        assert!(cmd.handle_message(&msg));
        assert!(cmd.get_client_messages().is_empty());
        assert!(!cmd.is_done());
        let msg = ":irc.example.com 501 luser :Unknown user mode: +Q"
            .parse::<Message>()
            .unwrap();
        assert!(cmd.handle_message(&msg));
        assert!(cmd.get_client_messages().is_empty());
        assert!(cmd.is_done());
        let output = cmd.get_output().unwrap();
        assert_eq!(
            output,
            SetUserModeOutput {
                new_mode: "+Zqw".parse::<ModeString>().unwrap(),
                umode_unknown_message: Some("Unknown user mode: +Q".to_owned()),
            }
        );
    }

    #[test]
    fn good_and_bad_modes_501_reply_first() {
        let target = "luser".parse::<ModeTarget>().unwrap();
        let mstr = "+Qq".parse::<ModeString>().unwrap();
        let mut cmd = SetUserMode::new(target, mstr);
        let outgoing = cmd
            .get_client_messages()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["MODE luser +Qq"]);
        let msg = ":irc.example.com 501 luser :Unknown user mode: +Q"
            .parse::<Message>()
            .unwrap();
        assert!(cmd.handle_message(&msg));
        assert!(cmd.get_client_messages().is_empty());
        assert!(!cmd.is_done());
        let msg = ":irc.example.com MODE luser +Zqw"
            .parse::<Message>()
            .unwrap();
        assert!(cmd.handle_message(&msg));
        assert!(cmd.get_client_messages().is_empty());
        assert!(cmd.is_done());
        let output = cmd.get_output().unwrap();
        assert_eq!(
            output,
            SetUserModeOutput {
                new_mode: "+Zqw".parse::<ModeString>().unwrap(),
                umode_unknown_message: Some("Unknown user mode: +Q".to_owned()),
            }
        );
    }
}
