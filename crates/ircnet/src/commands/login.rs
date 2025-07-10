use super::Command;
use irctext::{
    ClientMessage, ClientMessageParts, FinalParam, Message, Payload, Reply, ReplyParts,
    clientmsgs::{Mode, Nick, Pass, User},
    types::{ISupportParam, ModeString, Nickname, ReplyTarget, Username},
};
use std::time::Duration;
use thiserror::Error;

/// How long to wait for an optional `MODE` or `RPL_UMODEIS` (221) message
/// after receiving the MOTD
const MODE_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoginParams {
    pub password: FinalParam,
    pub nickname: Nickname,
    pub username: Username,
    pub realname: FinalParam,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Login {
    outgoing: Vec<ClientMessage>,
    state: State,
}

impl Login {
    pub fn new(params: LoginParams) -> Login {
        let pass = ClientMessage::from(Pass::new(params.password));
        let nick = ClientMessage::from(Nick::new(params.nickname));
        let user = ClientMessage::from(User::new(params.username, params.realname));
        Login {
            outgoing: vec![pass, nick, user],
            state: State::Start,
        }
    }

    fn error(&mut self, e: LoginError) {
        self.state = State::Done(Some(Err(e)));
    }
}

// Order of replies on successful login:
// - With SASL:
//     - RPL_LOGGEDIN (900)
//     - RPL_SASLSUCCESS (903)
// - RPL_WELCOME (001)
// - RPL_YOURHOST (002)
// - RPL_CREATED (003)
// - RPL_MYINFO (004)
// - one or more RPL_ISUPPORT (005)
// - "other numerics and messages"
// - optional: lusers:
//     - required: RPL_LUSERCLIENT (251), RPL_LUSERME (255)
//     - optional: RPL_LUSEROP (252), RPL_LUSERUNKNOWN (253), RPL_LUSERCHANNELS
//       (254), RPL_LOCALUSERS (265), RPL_GLOBALUSERS (266)
// - motd: one of:
//     - RPL_MOTDSTART (375), one or more RPL_MOTD (372), RPL_ENDOFMOTD (376)
//     - ERR_NOMOTD (422)
// - optional: mode := RPL_UMODEIS (221) or MODE

// Possible error replies on login:
//  - ERR_INPUTTOOLONG (417)
//  - ERR_UNKNOWNCOMMAND (421)
//      - When using SASL, this may be sent in reply to CAP if the server
//        doesn't support the command, in which case we should gracefully fall
//        back to plain login.
//  - ERR_ERRONEUSNICKNAME (432)
//  - ERR_NICKNAMEINUSE (433)
//  - ERR_NICKCOLLISION (436) ?
//  - ERR_PASSWDMISMATCH (464)
//  - ERR_YOUREBANNEDCREEP (465)
//  - With SASL:
//      - ERR_NICKLOCKED (902)
//      - ERR_SASLFAIL (904)
//      - ERR_SASLTOOLONG (905) ?
//      - ERR_SASLABORTED (906) ?
//      - ERR_SASLALREADY (907)

impl Command for Login {
    type Output = Result<LoginOutput, LoginError>;

    fn get_client_messages(&mut self) -> Vec<ClientMessage> {
        std::mem::take(&mut self.outgoing)
    }

    fn handle_message(&mut self, msg: &Message) -> bool {
        match &msg.payload {
            Payload::Reply(rpl) => {
                if rpl.is_error() && !matches!(rpl, Reply::NoMotd(_)) {
                    match rpl {
                        Reply::InputTooLong(r) => {
                            self.error(LoginError::InputTooLong {
                                message: r.message().to_string(),
                            });
                        }
                        Reply::UnknownCommand(r) => self.error(LoginError::UnknownCommand {
                            command: r.command().to_string(),
                            message: r.message().to_string(),
                        }),
                        Reply::ErroneousNickname(r) => self.error(LoginError::ErroneousNickname {
                            nickname: r.nickname().to_string(),
                            message: r.message().to_string(),
                        }),
                        Reply::NicknameInUse(r) => self.error(LoginError::NicknameInUse {
                            nickname: r.nickname().to_string(),
                            message: r.message().to_string(),
                        }),
                        Reply::NickCollision(r) => self.error(LoginError::NicknameCollision {
                            nickname: r.nickname().to_string(),
                            message: r.message().to_string(),
                        }),
                        Reply::PasswdMismatch(r) => {
                            self.error(LoginError::Password {
                                message: r.message().to_string(),
                            });
                        }
                        Reply::YoureBannedCreep(r) => {
                            self.error(LoginError::Banned {
                                message: r.message().to_string(),
                            });
                        }
                        unexpected => self.error(LoginError::UnexpectedError {
                            code: unexpected.code(),
                            reply: msg.to_string(),
                        }),
                    }
                    true
                } else {
                    match self.state.in_place(|state| state.handle_reply(rpl)) {
                        Ok(b) => b,
                        Err(e) => {
                            self.error(e);
                            true
                        }
                    }
                }
            }
            Payload::ClientMessage(climsg) => match climsg {
                ClientMessage::Mode(mode) => {
                    match self.state.in_place(|state| state.handle_mode(mode)) {
                        Ok(b) => b,
                        Err(e) => {
                            self.error(e);
                            true
                        }
                    }
                }
                ClientMessage::Ping(_) | ClientMessage::PrivMsg(_) | ClientMessage::Notice(_) => {
                    false
                }
                other => match self.state.handle_other(other) {
                    Ok(b) => b,
                    Err(e) => {
                        self.error(e);
                        true
                    }
                },
            },
        }
    }

    fn get_timeout(&mut self) -> Option<Duration> {
        if let State::AwaitingMode {
            ref mut timeout, ..
        } = self.state
        {
            timeout.take()
        } else {
            None
        }
    }

    fn handle_timeout(&mut self) {
        let state = std::mem::replace(&mut self.state, State::Void);
        self.state = match state {
            State::AwaitingMode {
                timeout: None,
                output,
            } => State::Done(Some(Ok(output))),
            other => other,
        };
    }

    fn is_done(&self) -> bool {
        matches!(self.state, State::Done(_))
    }

    fn get_output(&mut self) -> Self::Output {
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
    Got001 {
        my_nick: Nickname,
    },
    Got002 {
        my_nick: Nickname,
    },
    Got003 {
        my_nick: Nickname,
    },
    Got004(LoginOutput),
    Got005(LoginOutput),
    Lusers(LoginOutput),
    Motd(LoginOutput),
    AwaitingMode {
        timeout: Option<Duration>,
        output: LoginOutput,
    },
    Done(Option<Result<LoginOutput, LoginError>>),
    Void,
}

impl State {
    fn in_place<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(Self) -> (State, T),
    {
        let state = std::mem::replace(self, State::Void);
        let (state, r) = f(state);
        *self = state;
        r
    }

    fn handle_reply(self, rpl: &Reply) -> (State, Result<bool, LoginError>) {
        match (self, rpl) {
            (st @ State::Start, Reply::Welcome(r)) => {
                if let ReplyTarget::Nick(nick) = r.client() {
                    let my_nick = nick.clone();
                    (State::Got001 { my_nick }, Ok(true))
                } else {
                    (st, Err(LoginError::StarWelcome))
                }
            }
            (State::Got001 { my_nick }, Reply::YourHost(_)) => {
                (State::Got002 { my_nick }, Ok(true))
            }
            (State::Got002 { my_nick }, Reply::Created(_)) => (State::Got003 { my_nick }, Ok(true)),
            (State::Got003 { my_nick }, Reply::MyInfo(r)) => {
                let server_info = ServerInfo {
                    server_name: r.servername().to_owned(),
                    version: r.version().to_owned(),
                    user_modes: r.available_user_modes().to_owned(),
                    channel_modes: r.available_channel_modes().to_owned(),
                    param_channel_modes: r.channel_modes_with_param().map(ToOwned::to_owned),
                };
                let output = LoginOutput {
                    my_nick,
                    server_info,
                    isupport: Vec::new(),
                    luser_stats: LuserStats::default(),
                    motd: None,
                    mode: None,
                };
                (State::Got004(output), Ok(true))
            }
            (State::Got004(mut output) | State::Got005(mut output), Reply::ISupport(r)) => {
                output.isupport.extend(r.tokens().iter().cloned());
                (State::Got005(output), Ok(true))
            }
            (State::Got005(output) | State::Lusers(output), Reply::LuserClient(_)) => {
                (State::Lusers(output), Ok(true))
            }
            (State::Got005(mut output) | State::Lusers(mut output), Reply::LuserOp(r)) => {
                output.luser_stats.operators = Some(r.ops());
                (State::Lusers(output), Ok(true))
            }
            (State::Got005(mut output) | State::Lusers(mut output), Reply::LuserUnknown(r)) => {
                output.luser_stats.unknown_connections = Some(r.connections());
                (State::Lusers(output), Ok(true))
            }
            (State::Got005(mut output) | State::Lusers(mut output), Reply::LuserChannels(r)) => {
                output.luser_stats.channels = Some(r.channels());
                (State::Lusers(output), Ok(true))
            }
            (State::Got005(output) | State::Lusers(output), Reply::LuserMe(_)) => {
                (State::Lusers(output), Ok(true))
            }
            (State::Got005(mut output) | State::Lusers(mut output), Reply::LocalUsers(r)) => {
                output.luser_stats.local_clients = r.current_users();
                output.luser_stats.max_local_clients = r.max_users();
                (State::Lusers(output), Ok(true))
            }
            (State::Got005(mut output) | State::Lusers(mut output), Reply::GlobalUsers(r)) => {
                output.luser_stats.global_clients = r.current_users();
                output.luser_stats.max_global_clients = r.max_users();
                (State::Lusers(output), Ok(true))
            }
            (State::Got005(mut output) | State::Lusers(mut output), Reply::MotdStart(r)) => {
                output.motd = Some(r.message().to_owned());
                (State::Motd(output), Ok(true))
            }
            (State::Got005(mut output) | State::Lusers(mut output), Reply::NoMotd(r)) => {
                output.motd = Some(r.message().to_owned());
                (
                    State::AwaitingMode {
                        output,
                        timeout: Some(MODE_TIMEOUT),
                    },
                    Ok(true),
                )
            }
            (st @ State::Got005(_), _) => (st, Ok(false)), // Accept "other numerics and messages" after RPL_ISUPPORT
            (State::Motd(mut output), Reply::Motd(r)) => {
                if let Some(s) = output.motd.as_mut() {
                    s.push('\n');
                    s.push_str(r.message());
                }
                (State::Motd(output), Ok(true))
            }
            (State::Motd(mut output), Reply::EndOfMotd(r)) => {
                if let Some(s) = output.motd.as_mut() {
                    s.push('\n');
                    s.push_str(r.message());
                }
                (
                    State::AwaitingMode {
                        output,
                        timeout: Some(MODE_TIMEOUT),
                    },
                    Ok(true),
                )
            }
            (State::AwaitingMode { mut output, .. }, Reply::UModeIs(r)) => {
                let ms = r.user_modes();
                let ms = if ms.starts_with(['+', '-']) {
                    ms.to_owned()
                } else {
                    format!("+{ms}")
                };
                let Ok(modestring) = ms.parse::<ModeString>() else {
                    return (
                        State::Void,
                        Err(LoginError::InvalidMode {
                            msg: r.to_irc_line(),
                        }),
                    );
                };
                output.mode = Some(modestring);
                (State::Done(Some(Ok(output))), Ok(true))
            }
            (st @ (State::Done(_) | State::Void), _) => (st, Ok(false)),
            (st, other) => {
                let expecting = st.expecting();
                let msg = other.to_irc_line();
                (st, Err(LoginError::Unexpected { expecting, msg }))
            }
        }
    }

    fn handle_mode(self, mode: &Mode) -> (State, Result<bool, LoginError>) {
        match self {
            State::AwaitingMode { mut output, .. } => {
                output.mode = mode.modestring().cloned();
                (State::Done(Some(Ok(output))), Ok(true))
            }
            st => {
                let expecting = st.expecting();
                let msg = mode.to_irc_line();
                (st, Err(LoginError::Unexpected { expecting, msg }))
            }
        }
    }

    fn handle_other(&self, climsg: &ClientMessage) -> Result<bool, LoginError> {
        if matches!(self, State::Got005(_)) {
            // Accept "other numerics and messages" after RPL_ISUPPORT
            Ok(false)
        } else {
            let expecting = self.expecting();
            let msg = climsg.to_irc_line();
            Err(LoginError::Unexpected { expecting, msg })
        }
    }

    fn expecting(&self) -> &'static str {
        match self {
            State::Start => "RPL_WELCOME (001) reply",
            State::Got001 { .. } => "RPL_YOURHOST (002) reply",
            State::Got002 { .. } => "RPL_CREATED (003) reply",
            State::Got003 { .. } => "RPL_MYINFO (004) reply",
            State::Got004(_) => "RPL_ISUPPORT (005) reply",
            State::Got005(_) => "RPL_ISUPPORT (005) reply, LUSERS reply, or MOTD reply",
            State::Lusers(_) => "LUSERS reply or MOTD reply",
            State::Motd(_) => "RPL_MOTD (372) or RPL_ENDOFMOTD (376)",
            State::AwaitingMode { .. } => "MODE or RPL_UMODEIS (221)",
            State::Done(_) => "nothing",
            State::Void => "nothing",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoginOutput {
    // SASL: CAP LS
    my_nick: Nickname,
    server_info: ServerInfo,
    isupport: Vec<ISupportParam>,
    luser_stats: LuserStats,
    motd: Option<String>, // None if the server reports no MOTD was set
    mode: Option<ModeString>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerInfo {
    server_name: String,
    version: String,
    user_modes: String,
    channel_modes: String,
    param_channel_modes: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LuserStats {
    operators: Option<u64>,
    unknown_connections: Option<u64>,
    channels: Option<u64>,
    local_clients: Option<u64>,
    max_local_clients: Option<u64>,
    global_clients: Option<u64>,
    max_global_clients: Option<u64>,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum LoginError {
    #[error("login failed due to overly-long input line: {message:?}")]
    InputTooLong { message: String },
    #[error("login failed because server does not recognize {command:?} command: {message:?}")]
    UnknownCommand { command: String, message: String },
    #[error("login failed because server rejected nickname {nickname:?} as invalid: {message:?}")]
    ErroneousNickname { nickname: String, message: String },
    #[error("login failed because {nickname:?} is already in use: {message:?}")]
    NicknameInUse { nickname: String, message: String },
    #[error("login failed because {nickname:?} had a collision: {message:?}")]
    NicknameCollision { nickname: String, message: String },
    #[error("login failed because password was rejected: {message:?}")]
    Password { message: String },
    #[error("login failed because client is banned: {message:?}")]
    Banned { message: String },
    #[error("login failed with unexpected error reply {code:03}: {reply:?}")]
    UnexpectedError { code: u16, reply: String },
    #[error("login failed because RPL_WELCOME was addressed to * instead of client nickname")]
    StarWelcome,
    #[error(
        "login failed because server sent unexpected message: expecting {expecting}, got {msg:?}"
    )]
    Unexpected {
        expecting: &'static str,
        msg: String,
    },
    #[error("login failed because server sent unparseable mode string in RPL_UMODEIS: {msg:?}")]
    InvalidMode { msg: String },
}
