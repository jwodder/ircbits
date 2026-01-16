use super::Command;
use irctext::{
    ClientMessage, ClientMessageParts, Message, Payload, Reply, ReplyParts, TrailingParam,
    clientmsgs::{
        Authenticate, Cap, CapEnd, CapLsRequest, CapReq, Capability, CapabilityRequest,
        CapabilityValue, Mode, Nick, Pass, User,
    },
    types::{ISupportParam, ModeString, Nickname, ReplyTarget, Username},
};
use itertools::Itertools; // join
use replace_with::{replace_with, replace_with_and_return};
use std::time::Duration;
use thiserror::Error;

/// How long to wait for an optional `MODE` or `RPL_UMODEIS` (221) message
/// after receiving the MOTD
const MODE_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LoginParams {
    pub password: TrailingParam,
    pub nickname: Nickname,
    pub username: Username,
    pub realname: TrailingParam,
    #[cfg_attr(feature = "serde", serde(default = "default_sasl"))]
    pub sasl: bool,
}

#[cfg(feature = "serde")]
fn default_sasl() -> bool {
    true
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Login {
    outgoing: Vec<ClientMessage>,
    auth_msgs: Vec<ClientMessage>,
    state: State,
}

impl Login {
    pub fn new(params: LoginParams) -> Login {
        let auth_msgs =
            Authenticate::new_plain_sasl(&params.nickname, &params.nickname, &params.password)
                .into_iter()
                .map(ClientMessage::from)
                .collect::<Vec<_>>();
        let mut outgoing = Vec::with_capacity(4);
        if params.sasl {
            outgoing.push(ClientMessage::from(CapLsRequest::new_with_version(302)));
        }
        outgoing.push(ClientMessage::from(Pass::new(params.password)));
        outgoing.push(ClientMessage::from(Nick::new(params.nickname)));
        outgoing.push(ClientMessage::from(User::new(
            params.username,
            params.realname,
        )));
        Login {
            outgoing,
            auth_msgs,
            state: if params.sasl {
                State::Start
            } else {
                State::Awaiting001 { capabilities: None }
            },
        }
    }
}

// Order of replies on successful login:
// - With SASL:
//     - S: `CAP * LS :…`, possibly preceded by zero or more `CAP * LS * :…`
//          - If "PLAIN" isn't in sasl, send CAP END and await RPL_WELCOME
//     - C: CAP REQ sasl
//     - S: CAP * ACK sasl
//          - Or `CAP * NAK sasl`, in which case we error out
//     - C: AUTHENTICATE PLAIN
//     - S: AUTHENTICATE +
//          - Or ERR_SASLMECHS (908)
//     - C: AUTHENTICATE {base64("{nickname}\0{nickname}\0{password}")}
//     - S: RPL_LOGGEDIN (900)
//     - S: RPL_SASLSUCCESS (903)
// - Without SASL:
//     - Either 421 for CAP or nothing
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
//     - nonstandard: RPL_STATSCONN (250)
// - motd: one of:
//     - RPL_MOTDSTART (375), one or more RPL_MOTD (372), RPL_ENDOFMOTD (376)
//     - ERR_NOMOTD (422)
// - optional: mode := RPL_UMODEIS (221) or MODE

// Possible error replies on login:
//  - ERROR message
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
    type Output = LoginOutput;
    type Error = LoginError;

    fn get_client_messages(&mut self) -> Vec<ClientMessage> {
        std::mem::take(&mut self.outgoing)
    }

    fn handle_message(&mut self, msg: &Message) -> bool {
        match &msg.payload {
            Payload::Reply(rpl) => {
                if rpl.is_error() && !matches!(rpl, Reply::NoMotd(_)) {
                    if self.state == State::Start
                        && let Reply::UnknownCommand(r) = rpl
                        && r.command() == "CAP"
                    {
                        self.state = State::Awaiting001 { capabilities: None };
                        return true;
                    }
                    let e = match rpl {
                        Reply::InputTooLong(r) => LoginError::InputTooLong {
                            message: r.message().to_string(),
                        },
                        Reply::UnknownCommand(r) => LoginError::UnknownCommand {
                            command: r.command().to_string(),
                            message: r.message().to_string(),
                        },
                        Reply::ErroneousNickname(r) => LoginError::ErroneousNickname {
                            nickname: r.nickname().to_string(),
                            message: r.message().to_string(),
                        },
                        Reply::NicknameInUse(r) => LoginError::NicknameInUse {
                            nickname: r.nickname().to_string(),
                            message: r.message().to_string(),
                        },
                        Reply::NickCollision(r) => LoginError::NicknameCollision {
                            nickname: r.nickname().to_string(),
                            message: r.message().to_string(),
                        },
                        Reply::PasswdMismatch(r) => LoginError::Password {
                            message: r.message().to_string(),
                        },
                        Reply::YoureBannedCreep(r) => LoginError::Banned {
                            message: r.message().to_string(),
                        },
                        Reply::NickLocked(r) => LoginError::NickLocked {
                            message: r.message().to_string(),
                        },
                        Reply::SaslFail(r) => LoginError::SaslFail {
                            message: r.message().to_string(),
                        },
                        Reply::SaslAlready(r) => LoginError::SaslAlready {
                            message: r.message().to_string(),
                        },
                        Reply::SaslMechs(r) => LoginError::SaslMechs {
                            message: format!("{} {}", r.mechanisms(), r.message()),
                        },
                        unexpected => LoginError::UnexpectedError {
                            code: unexpected.code(),
                            reply: msg.to_string(),
                        },
                    };
                    self.state = State::error(e);
                    true
                } else {
                    let (b, msgs) = self.state.handle_reply(rpl);
                    self.outgoing.extend(msgs);
                    b
                }
            }
            Payload::ClientMessage(climsg) => match climsg {
                ClientMessage::Error(err) => {
                    self.state = State::error(LoginError::ErrorMessage {
                        reason: err.reason().to_string(),
                    });
                    true
                }
                ClientMessage::Mode(mode) => self.state.handle_mode(mode),
                ClientMessage::Ping(_) | ClientMessage::PrivMsg(_) | ClientMessage::Notice(_) => {
                    false
                }
                ClientMessage::Cap(cap) => {
                    if let Some(msg) = self.state.handle_cap(cap) {
                        self.outgoing.push(msg);
                    }
                    true
                }
                ClientMessage::Authenticate(auth) => {
                    if self.state.handle_auth(auth) {
                        let msgs = std::mem::take(&mut self.auth_msgs);
                        self.outgoing.extend(msgs);
                    }
                    true
                }
                other => self.state.handle_other(other),
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

    fn get_output(&mut self) -> Result<LoginOutput, LoginError> {
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
    ListingCaps {
        capabilities: Vec<(Capability, Option<CapabilityValue>)>,
    },
    AwaitingAck {
        capabilities: Vec<(Capability, Option<CapabilityValue>)>,
    },
    SentMechanism {
        capabilities: Vec<(Capability, Option<CapabilityValue>)>,
    },
    SentAuth {
        capabilities: Vec<(Capability, Option<CapabilityValue>)>,
    },
    Got900 {
        capabilities: Vec<(Capability, Option<CapabilityValue>)>,
    },
    Awaiting001 {
        capabilities: Option<Vec<(Capability, Option<CapabilityValue>)>>,
    },
    Got001 {
        my_nick: Nickname,
        welcome_msg: String,
        capabilities: Option<Vec<(Capability, Option<CapabilityValue>)>>,
    },
    Got002 {
        my_nick: Nickname,
        welcome_msg: String,
        yourhost_msg: String,
        capabilities: Option<Vec<(Capability, Option<CapabilityValue>)>>,
    },
    Got003 {
        my_nick: Nickname,
        welcome_msg: String,
        yourhost_msg: String,
        created_msg: String,
        capabilities: Option<Vec<(Capability, Option<CapabilityValue>)>>,
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
    fn error(e: LoginError) -> State {
        State::Done(Some(Err(e)))
    }

    fn handle_reply(&mut self, rpl: &Reply) -> (bool, Option<ClientMessage>) {
        replace_with_and_return(
            self,
            || State::Void,
            |state| match (state, rpl) {
                (State::Start, Reply::Welcome(r)) => {
                    if let ReplyTarget::Nick(nick) = r.client() {
                        let my_nick = nick.clone();
                        (
                            (true, None),
                            State::Got001 {
                                my_nick,
                                welcome_msg: r.message().to_owned(),
                                capabilities: None,
                            },
                        )
                    } else {
                        ((true, None), State::error(LoginError::StarWelcome))
                    }
                }
                (State::SentAuth { capabilities }, Reply::LoggedIn(_)) => {
                    ((true, None), State::Got900 { capabilities })
                }
                (State::Got900 { capabilities }, Reply::SaslSuccess(_)) => (
                    (true, Some(ClientMessage::from(CapEnd))),
                    State::Awaiting001 {
                        capabilities: Some(capabilities),
                    },
                ),
                (State::Awaiting001 { capabilities }, Reply::Welcome(r)) => {
                    if let ReplyTarget::Nick(nick) = r.client() {
                        let my_nick = nick.clone();
                        (
                            (true, None),
                            State::Got001 {
                                my_nick,
                                welcome_msg: r.message().to_owned(),
                                capabilities,
                            },
                        )
                    } else {
                        ((true, None), State::error(LoginError::StarWelcome))
                    }
                }
                (
                    State::Got001 {
                        my_nick,
                        welcome_msg,
                        capabilities,
                    },
                    Reply::YourHost(r),
                ) => (
                    (true, None),
                    State::Got002 {
                        my_nick,
                        welcome_msg,
                        yourhost_msg: r.message().to_owned(),
                        capabilities,
                    },
                ),
                (
                    State::Got002 {
                        my_nick,
                        welcome_msg,
                        yourhost_msg,
                        capabilities,
                    },
                    Reply::Created(r),
                ) => (
                    (true, None),
                    State::Got003 {
                        my_nick,
                        welcome_msg,
                        yourhost_msg,
                        created_msg: r.message().to_owned(),
                        capabilities,
                    },
                ),
                (
                    State::Got003 {
                        my_nick,
                        welcome_msg,
                        yourhost_msg,
                        created_msg,
                        capabilities,
                    },
                    Reply::MyInfo(r),
                ) => {
                    let server_info = ServerInfo {
                        name: r.servername().to_owned(),
                        version: r.version().to_owned(),
                        user_modes: r.available_user_modes().to_owned(),
                        channel_modes: r.available_channel_modes().to_owned(),
                        param_channel_modes: r.channel_modes_with_param().map(ToOwned::to_owned),
                    };
                    let output = LoginOutput {
                        capabilities,
                        my_nick,
                        welcome_msg,
                        yourhost_msg,
                        created_msg,
                        server_info,
                        isupport: Vec::new(),
                        luser_stats: LuserStats::default(),
                        motd: None,
                        mode: None,
                    };
                    ((true, None), State::Got004(output))
                }
                (State::Got004(mut output) | State::Got005(mut output), Reply::ISupport(r)) => {
                    output.isupport.extend(r.tokens().iter().cloned());
                    ((true, None), State::Got005(output))
                }
                (State::Got005(output) | State::Lusers(output), Reply::StatsConn(_)) => {
                    ((true, None), State::Lusers(output))
                }
                (State::Got005(output) | State::Lusers(output), Reply::LuserClient(_)) => {
                    ((true, None), State::Lusers(output))
                }
                (State::Got005(mut output) | State::Lusers(mut output), Reply::LuserOp(r)) => {
                    output.luser_stats.operators = Some(r.ops());
                    ((true, None), State::Lusers(output))
                }
                (State::Got005(mut output) | State::Lusers(mut output), Reply::LuserUnknown(r)) => {
                    output.luser_stats.unknown_connections = Some(r.connections());
                    ((true, None), State::Lusers(output))
                }
                (
                    State::Got005(mut output) | State::Lusers(mut output),
                    Reply::LuserChannels(r),
                ) => {
                    output.luser_stats.channels = Some(r.channels());
                    ((true, None), State::Lusers(output))
                }
                (State::Got005(output) | State::Lusers(output), Reply::LuserMe(_)) => {
                    ((true, None), State::Lusers(output))
                }
                (State::Got005(mut output) | State::Lusers(mut output), Reply::LocalUsers(r)) => {
                    output.luser_stats.local_clients = r.current_users();
                    output.luser_stats.max_local_clients = r.max_users();
                    ((true, None), State::Lusers(output))
                }
                (State::Got005(mut output) | State::Lusers(mut output), Reply::GlobalUsers(r)) => {
                    output.luser_stats.global_clients = r.current_users();
                    output.luser_stats.max_global_clients = r.max_users();
                    ((true, None), State::Lusers(output))
                }
                (State::Got005(mut output) | State::Lusers(mut output), Reply::MotdStart(r)) => {
                    output.motd = Some(r.message().to_owned());
                    ((true, None), State::Motd(output))
                }
                (State::Got005(mut output) | State::Lusers(mut output), Reply::NoMotd(r)) => {
                    output.motd = Some(r.message().to_owned());
                    (
                        (true, None),
                        State::AwaitingMode {
                            output,
                            timeout: Some(MODE_TIMEOUT),
                        },
                    )
                }
                (st @ State::Got005(_), _) => ((false, None), st), // Accept "other numerics and messages" after RPL_ISUPPORT
                (State::Motd(mut output), Reply::Motd(r)) => {
                    if let Some(s) = output.motd.as_mut() {
                        s.push('\n');
                        s.push_str(r.message());
                    }
                    ((true, None), State::Motd(output))
                }
                (State::Motd(mut output), Reply::EndOfMotd(r)) => {
                    if let Some(s) = output.motd.as_mut() {
                        s.push('\n');
                        s.push_str(r.message());
                    }
                    (
                        (true, None),
                        State::AwaitingMode {
                            output,
                            timeout: Some(MODE_TIMEOUT),
                        },
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
                            (true, None),
                            State::error(LoginError::InvalidMode {
                                msg: r.to_irc_line(),
                            }),
                        );
                    };
                    output.mode = Some(modestring);
                    ((true, None), State::Done(Some(Ok(output))))
                }
                (st @ State::Done(_), _) => ((false, None), st),
                (State::Void, _) => panic!("handle_reply() called on Void login state"),
                (st, other) => {
                    let expecting = st.expecting();
                    let msg = other.to_irc_line();
                    (
                        (true, None),
                        State::error(LoginError::Unexpected { expecting, msg }),
                    )
                }
            },
        )
    }

    fn handle_mode(&mut self, mode: &Mode) -> bool {
        replace_with(
            self,
            || State::Void,
            |state| match state {
                State::AwaitingMode { mut output, .. } => {
                    output.mode = mode.modestring().cloned();
                    State::Done(Some(Ok(output)))
                }
                State::Void => panic!("handle_mode() called on Void login state"),
                st => {
                    let expecting = st.expecting();
                    let msg = mode.to_irc_line();
                    State::error(LoginError::Unexpected { expecting, msg })
                }
            },
        );
        true
    }

    fn handle_cap(&mut self, cap: &Cap) -> Option<ClientMessage> {
        replace_with_and_return(
            self,
            || State::Void,
            |state| match (state, cap) {
                (State::Start, Cap::LsResponse(m)) => {
                    let capabilities = m.capabilities.clone();
                    if m.continued {
                        (None, State::ListingCaps { capabilities })
                    } else if has_plain_sasl(&capabilities) {
                        let cap_req = ClientMessage::from(CapReq {
                            capabilities: vec![
                                "sasl"
                                    .parse::<CapabilityRequest>()
                                    .expect(r#""sasl" should be valid capability request"#),
                            ],
                        });
                        (Some(cap_req), State::AwaitingAck { capabilities })
                    } else {
                        let cap_end = ClientMessage::from(CapEnd);
                        (
                            Some(cap_end),
                            State::Awaiting001 {
                                capabilities: Some(capabilities),
                            },
                        )
                    }
                }
                (State::ListingCaps { mut capabilities }, Cap::LsResponse(m)) => {
                    capabilities.extend(m.capabilities.clone());
                    if m.continued {
                        (None, State::ListingCaps { capabilities })
                    } else if has_plain_sasl(&capabilities) {
                        let cap_req = ClientMessage::from(CapReq {
                            capabilities: vec![
                                "sasl"
                                    .parse::<CapabilityRequest>()
                                    .expect(r#""sasl" should be valid capability request"#),
                            ],
                        });
                        (Some(cap_req), State::AwaitingAck { capabilities })
                    } else {
                        let cap_end = ClientMessage::from(CapEnd);
                        (
                            Some(cap_end),
                            State::Awaiting001 {
                                capabilities: Some(capabilities),
                            },
                        )
                    }
                }
                (State::AwaitingAck { capabilities }, Cap::Ack(m)) => {
                    if m.capabilities
                        .iter()
                        .any(|c| c.capability == "sasl" && !c.disable)
                    {
                        let auth_plain = ClientMessage::from(Authenticate::new(
                            "PLAIN"
                                .parse::<TrailingParam>()
                                .expect(r#""PLAIN" should be valid trailing param"#),
                        ));
                        (Some(auth_plain), State::SentMechanism { capabilities })
                    } else {
                        (
                            None,
                            State::error(LoginError::BadAckNak {
                                requested: "sasl",
                                cmd: "ACK",
                                acked: m.capabilities.iter().join(" "),
                            }),
                        )
                    }
                }
                (State::AwaitingAck { .. }, Cap::Nak(m)) => {
                    if m.capabilities.iter().any(|c| c == "sasl") {
                        (None, State::error(LoginError::SaslNacked))
                    } else {
                        (
                            None,
                            State::error(LoginError::BadAckNak {
                                requested: "sasl",
                                cmd: "NAK",
                                acked: m.capabilities.iter().join(" "),
                            }),
                        )
                    }
                }
                (state, other) => {
                    let expecting = state.expecting();
                    let msg = other.to_irc_line();
                    (
                        None,
                        State::error(LoginError::Unexpected { expecting, msg }),
                    )
                }
            },
        )
    }

    fn handle_auth(&mut self, auth: &Authenticate) -> bool {
        replace_with_and_return(
            self,
            || State::Void,
            |state| {
                match (state, auth.parameter().as_str()) {
                    (State::SentMechanism { capabilities }, "+") => {
                        (true, State::SentAuth { capabilities })
                        // Caller should now send auth_msgs
                    }
                    (state, _) => {
                        let expecting = state.expecting();
                        let msg = auth.to_irc_line();
                        (
                            false,
                            State::error(LoginError::Unexpected { expecting, msg }),
                        )
                    }
                }
            },
        )
    }

    fn handle_other(&mut self, climsg: &ClientMessage) -> bool {
        match self {
            State::Got005(_) => {
                // Accept "other numerics and messages" after RPL_ISUPPORT
                false
            }
            State::Void => panic!("handle_other() called on Void login state"),
            _ => {
                let expecting = self.expecting();
                let msg = climsg.to_irc_line();
                *self = State::error(LoginError::Unexpected { expecting, msg });
                true
            }
        }
    }

    fn expecting(&self) -> &'static str {
        match self {
            State::Start => r#""CAP * LS" response or RPL_WELCOME (001) reply"#,
            State::ListingCaps { .. } => r#""CAP * LS" response"#,
            State::AwaitingAck { .. } => r#""CAP * ACK" response"#,
            State::SentMechanism { .. } => r#""AUTHENTICATE +" response"#,
            State::SentAuth { .. } => "RPL_LOGGEDIN (900) reply",
            State::Got900 { .. } => "RPL_SASLSUCCESS (903) reply",
            State::Awaiting001 { .. } => "RPL_WELCOME (001) reply",
            State::Got001 { .. } => "RPL_YOURHOST (002) reply",
            State::Got002 { .. } => "RPL_CREATED (003) reply",
            State::Got003 { .. } => "RPL_MYINFO (004) reply",
            State::Got004(_) => "RPL_ISUPPORT (005) reply",
            State::Got005(_) => "RPL_ISUPPORT (005) reply, LUSERS reply, or MOTD reply",
            State::Lusers(_) => "LUSERS reply or MOTD reply",
            State::Motd(_) => "RPL_MOTD (372) or RPL_ENDOFMOTD (376)",
            State::AwaitingMode { .. } => "MODE or RPL_UMODEIS (221)",
            State::Done(_) => "nothing",
            State::Void => panic!("expecting() called on Void login state"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoginOutput {
    /// If the server supports capability negotiation, this field contains its
    /// supported capabilities and their optional values
    pub capabilities: Option<Vec<(Capability, Option<CapabilityValue>)>>,

    /// The nickname with which the command logged into IRC as given in the
    /// `RPL_WELCOME` reply
    pub my_nick: Nickname,

    /// The message given in the `RPL_WELCOME` reply
    pub welcome_msg: String,

    /// The message given in the `RPL_YOURHOST` reply
    pub yourhost_msg: String,

    /// The message given in the `RPL_CREATED` reply
    pub created_msg: String,

    /// Details about the IRC server as given in the `RPL_MYINFO` reply
    pub server_info: ServerInfo,

    /// Features advertised by the server via the parameters of the
    /// `RPL_ISUPPORT` reply
    pub isupport: Vec<ISupportParam>,

    /// Server statistics about users as supplied in the response to an
    /// optional implicit `LUSERS` command upon connection
    pub luser_stats: LuserStats,

    /// The server's message of the day, or `None` if no MOTD is set
    pub motd: Option<String>,

    /// The user's client modes, or `None` if the server did not report the
    /// mode upon login
    pub mode: Option<ModeString>,
}

/// Details about the IRC server as given in the `RPL_MYINFO` reply
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerInfo {
    /// The name of the server
    pub name: String,

    /// The server's software version
    pub version: String,

    /// The available user modes
    pub user_modes: String,

    /// The available channel modes that do not take parameters
    pub channel_modes: String,

    /// The available channel modes that take parameters, or `None` if there
    /// are none
    pub param_channel_modes: Option<String>,
}

/// Server statistics about users as supplied in the response to an optional
/// implicit `LUSERS` command upon connection
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LuserStats {
    /// The number of IRC operators connected to the server, or `None` if not
    /// given
    pub operators: Option<u64>,

    /// The number of connections to the server that are currently in an
    /// unknown state, or `None` if not given
    pub unknown_connections: Option<u64>,

    /// The number of channels that currently exist on the server, or `None` if
    /// not given
    pub channels: Option<u64>,

    /// The number of clients currently directly connected to this server, or
    /// `None` if not given
    pub local_clients: Option<u64>,

    /// The maximum number of clients ever directly connected to this server at
    /// one time, or `None` if not given
    pub max_local_clients: Option<u64>,

    /// The number of clients currently globally connected to this server, or
    /// `None` if not given
    pub global_clients: Option<u64>,

    /// The maximum number of clients ever globally connected to this server at
    /// one time, or `None` if not given
    pub max_global_clients: Option<u64>,
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
    #[error("login failed because nickname is locked: {message:?}")]
    NickLocked { message: String },
    #[error("login failed because SASL authentication failed: {message:?}")]
    SaslFail { message: String },
    #[error(
        "login failed because SASL authentication attempted while already logged in: {message:?}"
    )]
    SaslAlready { message: String },
    #[error("login failed because server rejected SASL PLAIN authentication: {message:?}")]
    SaslMechs { message: String },
    #[error("login failed with unexpected error reply {code:03}: {reply:?}")]
    UnexpectedError { code: u16, reply: String },
    #[error("server sent ERROR message during login: {reason:?}")]
    ErrorMessage { reason: String },
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
    #[error(
        r#"login failed because server responded to "CAP REQ {requested}" with inapplicable "CAP * {cmd} :{acked}"#
    )]
    BadAckNak {
        requested: &'static str,
        cmd: &'static str,
        acked: String,
    },
    #[error("login failed because server NAKed our request to enable SASL")]
    SaslNacked,
}

// Returns true if there's a "sasl" cap and it either lacks a value or has
// "PLAIN" in its value
fn has_plain_sasl(caps: &[(Capability, Option<CapabilityValue>)]) -> bool {
    caps.iter()
        .filter_map(|(key, value)| (key == "sasl").then_some(value.as_ref()))
        .next_back()
        .is_some_and(|m| m.is_none_or(|mechs| mechs.split(',').any(|m| m == "PLAIN")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_login() {
        let params = LoginParams {
            password: "hunter2".parse::<TrailingParam>().unwrap(),
            nickname: "jwodder".parse::<Nickname>().unwrap(),
            username: "jwuser".parse::<Username>().unwrap(),
            realname: "Just this guy, you know?".parse::<TrailingParam>().unwrap(),
            sasl: false,
        };
        let mut cmd = Login::new(params);
        let outgoing = cmd.get_client_messages();
        let outgoing = outgoing
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(
            outgoing,
            [
                "PASS :hunter2",
                "NICK jwodder",
                "USER jwuser 0 * :Just this guy, you know?"
            ]
        );

        let notices = [
            ":molybdenum.libera.chat NOTICE * :*** Checking Ident",
            ":molybdenum.libera.chat NOTICE * :*** Looking up your hostname...",
            ":molybdenum.libera.chat NOTICE * :*** Couldn't look up your hostname",
            ":molybdenum.libera.chat NOTICE * :*** No Ident response",
        ];
        for m in notices {
            let msg = m.parse::<Message>().unwrap();
            assert!(!cmd.handle_message(&msg));
            assert!(cmd.get_client_messages().is_empty());
            assert!(!cmd.is_done());
        }

        let incoming = [
            ":molybdenum.libera.chat 001 jwodder :Welcome to the Libera.Chat Internet Relay Chat Network jwodder",
            ":molybdenum.libera.chat 002 jwodder :Your host is molybdenum.libera.chat[2607:5300:205:300::ae0/6697], running version solanum-1.0-dev",
            ":molybdenum.libera.chat 003 jwodder :This server was created Thu Jul 18 2024 at 16:57:02 UTC",
            ":molybdenum.libera.chat 004 jwodder molybdenum.libera.chat solanum-1.0-dev DGIMQRSZaghilopsuwz CFILMPQRSTbcefgijklmnopqrstuvz bkloveqjfI",
            ":molybdenum.libera.chat 005 jwodder ACCOUNTEXTBAN=a WHOX KNOCK MONITOR=100 ETRACE FNC SAFELIST ELIST=CMNTU CALLERID=g CHANTYPES=# EXCEPTS INVEX :are supported by this server",
            ":molybdenum.libera.chat 005 jwodder CHANMODES=eIbq,k,flj,CFLMPQRSTcgimnprstuz CHANLIMIT=#:250 PREFIX=(ov)@+ MAXLIST=bqeI:100 MODES=4 NETWORK=Libera.Chat STATUSMSG=@+ CASEMAPPING=rfc1459 NICKLEN=16 MAXNICKLEN=16 CHANNELLEN=50 TOPICLEN=390 :are supported by this server",
            ":molybdenum.libera.chat 005 jwodder DEAF=D TARGMAX=NAMES:1,LIST:1,KICK:1,WHOIS:1,PRIVMSG:4,NOTICE:4,ACCEPT:,MONITOR: EXTBAN=$,agjrxz :are supported by this server",
            ":molybdenum.libera.chat 251 jwodder :There are 62 users and 31502 invisible on 28 servers",
            ":molybdenum.libera.chat 252 jwodder 40 :IRC Operators online",
            ":molybdenum.libera.chat 253 jwodder 66 :unknown connection(s)",
            ":molybdenum.libera.chat 254 jwodder 22798 :channels formed",
            ":molybdenum.libera.chat 255 jwodder :I have 2700 clients and 1 servers",
            ":molybdenum.libera.chat 265 jwodder 2700 3071 :Current local users 2700, max 3071",
            ":molybdenum.libera.chat 266 jwodder 31564 34153 :Current global users 31564, max 34153",
            ":molybdenum.libera.chat 250 jwodder :Highest connection count: 3072 (3071 clients) (781421 connections received)",
            ":molybdenum.libera.chat 375 jwodder :- molybdenum.libera.chat Message of the Day - ",
            ":molybdenum.libera.chat 372 jwodder :- -[ Molybdenum ]-[ Atomic number: 42 ]-[ Chemical symbol: Mo ]-",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Molybdenum has been known throughout history, though its",
            ":molybdenum.libera.chat 372 jwodder :- properties meant that it was confused with lead ores. It was",
            ":molybdenum.libera.chat 372 jwodder :- first identified as a distinct element in 1778, meaning our",
            ":molybdenum.libera.chat 372 jwodder :- modern understanding of its existence reaches about as far",
            ":molybdenum.libera.chat 372 jwodder :- back as American independence.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Molybdenum has a very high melting point, 2623°C, the fifth-",
            ":molybdenum.libera.chat 372 jwodder :- highest temperature of any element.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- While pure molybdenum is only about as strong as tooth enamel,",
            ":molybdenum.libera.chat 372 jwodder :- it forms strong alloys that are resistant to corrosion and",
            ":molybdenum.libera.chat 372 jwodder :- stable under varying temperatures. As a result, these alloys",
            ":molybdenum.libera.chat 372 jwodder :- are used for metal armour plates, high-performance aircraft",
            ":molybdenum.libera.chat 372 jwodder :- parts, and pumps for molten metal flows.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- In its elemental form, it can be used as a fertiliser, and the",
            ":molybdenum.libera.chat 372 jwodder :- isotope molybdenum-99 is used as a source of technetium-99m,",
            ":molybdenum.libera.chat 372 jwodder :- an important medical radioisotope used for 3D bone scans and",
            ":molybdenum.libera.chat 372 jwodder :- the investigation of blood flow when diagnosing heart disease.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Welcome to Libera Chat, the IRC network for",
            ":molybdenum.libera.chat 372 jwodder :- free & open-source software and peer directed projects.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Use of Libera Chat is governed by our network policies.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- To reduce network abuses we perform open proxy checks",
            ":molybdenum.libera.chat 372 jwodder :- on hosts at connection time.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Please visit us in #libera for questions and support.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Website and documentation:  https://libera.chat",
            ":molybdenum.libera.chat 372 jwodder :- Webchat:                    https://web.libera.chat",
            ":molybdenum.libera.chat 372 jwodder :- Network policies:           https://libera.chat/policies",
            ":molybdenum.libera.chat 372 jwodder :- Email:                      support@libera.chat",
            ":molybdenum.libera.chat 376 jwodder :End of /MOTD command.",
        ];
        for m in incoming {
            let msg = m.parse::<Message>().unwrap();
            assert!(cmd.handle_message(&msg));
            assert!(cmd.get_client_messages().is_empty());
            assert!(!cmd.is_done());
        }

        let m = ":jwodder MODE jwodder :+Ziw";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        assert!(cmd.get_client_messages().is_empty());
        assert!(cmd.is_done());

        let output = cmd.get_output().unwrap();
        pretty_assertions::assert_eq!(
            output,
            LoginOutput {
                capabilities: None,
                my_nick: "jwodder".parse::<Nickname>().unwrap(),
                welcome_msg: "Welcome to the Libera.Chat Internet Relay Chat Network jwodder".into(),
                yourhost_msg: "Your host is molybdenum.libera.chat[2607:5300:205:300::ae0/6697], running version solanum-1.0-dev".into(),
                created_msg: "This server was created Thu Jul 18 2024 at 16:57:02 UTC".into(),
                server_info: ServerInfo {
                    name: "molybdenum.libera.chat".into(),
                    version: "solanum-1.0-dev".into(),
                    user_modes: "DGIMQRSZaghilopsuwz".into(),
                    channel_modes: "CFILMPQRSTbcefgijklmnopqrstuvz".into(),
                    param_channel_modes: Some("bkloveqjfI".to_owned()),
                },
                isupport: [
                    "ACCOUNTEXTBAN=a",
                    "WHOX",
                    "KNOCK",
                    "MONITOR=100",
                    "ETRACE",
                    "FNC",
                    "SAFELIST",
                    "ELIST=CMNTU",
                    "CALLERID=g",
                    "CHANTYPES=#",
                    "EXCEPTS",
                    "INVEX",
                    "CHANMODES=eIbq,k,flj,CFLMPQRSTcgimnprstuz",
                    "CHANLIMIT=#:250",
                    "PREFIX=(ov)@+",
                    "MAXLIST=bqeI:100",
                    "MODES=4",
                    "NETWORK=Libera.Chat",
                    "STATUSMSG=@+",
                    "CASEMAPPING=rfc1459",
                    "NICKLEN=16",
                    "MAXNICKLEN=16",
                    "CHANNELLEN=50",
                    "TOPICLEN=390",
                    "DEAF=D",
                    "TARGMAX=NAMES:1,LIST:1,KICK:1,WHOIS:1,PRIVMSG:4,NOTICE:4,ACCEPT:,MONITOR:",
                    "EXTBAN=$,agjrxz",
                ]
                .into_iter()
                .map(str::parse::<ISupportParam>)
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
                luser_stats: LuserStats {
                    operators: Some(40),
                    unknown_connections: Some(66),
                    channels: Some(22798),
                    local_clients: Some(2700),
                    max_local_clients: Some(3071),
                    global_clients: Some(31564),
                    max_global_clients: Some(34153),
                },
                motd: Some(
                    concat!(
                        "- molybdenum.libera.chat Message of the Day - \n",
                        "- -[ Molybdenum ]-[ Atomic number: 42 ]-[ Chemical symbol: Mo ]-\n",
                        "-  \n",
                        "- Molybdenum has been known throughout history, though its\n",
                        "- properties meant that it was confused with lead ores. It was\n",
                        "- first identified as a distinct element in 1778, meaning our\n",
                        "- modern understanding of its existence reaches about as far\n",
                        "- back as American independence.\n",
                        "-  \n",
                        "- Molybdenum has a very high melting point, 2623°C, the fifth-\n",
                        "- highest temperature of any element.\n",
                        "-  \n",
                        "- While pure molybdenum is only about as strong as tooth enamel,\n",
                        "- it forms strong alloys that are resistant to corrosion and\n",
                        "- stable under varying temperatures. As a result, these alloys\n",
                        "- are used for metal armour plates, high-performance aircraft\n",
                        "- parts, and pumps for molten metal flows.\n",
                        "-  \n",
                        "- In its elemental form, it can be used as a fertiliser, and the\n",
                        "- isotope molybdenum-99 is used as a source of technetium-99m,\n",
                        "- an important medical radioisotope used for 3D bone scans and\n",
                        "- the investigation of blood flow when diagnosing heart disease.\n",
                        "-  \n",
                        "-  \n",
                        "- Welcome to Libera Chat, the IRC network for\n",
                        "- free & open-source software and peer directed projects.\n",
                        "-  \n",
                        "- Use of Libera Chat is governed by our network policies.\n",
                        "-  \n",
                        "- To reduce network abuses we perform open proxy checks\n",
                        "- on hosts at connection time.\n",
                        "-  \n",
                        "- Please visit us in #libera for questions and support.\n",
                        "-  \n",
                        "- Website and documentation:  https://libera.chat\n",
                        "- Webchat:                    https://web.libera.chat\n",
                        "- Network policies:           https://libera.chat/policies\n",
                        "- Email:                      support@libera.chat\n",
                        "End of /MOTD command.",
                    )
                    .to_owned()
                ),
                mode: Some("+Ziw".parse::<ModeString>().unwrap()),
            }
        );
    }

    #[test]
    fn sasl_plain_login() {
        let params = LoginParams {
            password: "hunter2".parse::<TrailingParam>().unwrap(),
            nickname: "jwodder".parse::<Nickname>().unwrap(),
            username: "jwuser".parse::<Username>().unwrap(),
            realname: "Just this guy, you know?".parse::<TrailingParam>().unwrap(),
            sasl: true,
        };
        let mut cmd = Login::new(params);

        let outgoing = cmd
            .get_client_messages()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(
            outgoing,
            [
                "CAP LS 302",
                "PASS :hunter2",
                "NICK jwodder",
                "USER jwuser 0 * :Just this guy, you know?"
            ]
        );

        let notices = [
            ":molybdenum.libera.chat NOTICE * :*** Checking Ident",
            ":molybdenum.libera.chat NOTICE * :*** Looking up your hostname...",
            ":molybdenum.libera.chat NOTICE * :*** Couldn't look up your hostname",
            ":molybdenum.libera.chat NOTICE * :*** No Ident response",
        ];
        for m in notices {
            let msg = m.parse::<Message>().unwrap();
            assert!(!cmd.handle_message(&msg));
            assert!(cmd.get_client_messages().is_empty());
            assert!(!cmd.is_done());
        }

        let m = ":molybdenum.libera.chat CAP * LS :account-notify away-notify chghost extended-join multi-prefix sasl=ECDSA-NIST256P-CHALLENGE,EXTERNAL,PLAIN,SCRAM-SHA-512 tls account-tag cap-notify echo-message server-time solanum.chat/identify-msg solanum.chat/oper solanum.chat/realhost";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        let outgoing = cmd
            .get_client_messages()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["CAP REQ :sasl"]);
        assert!(!cmd.is_done());

        let m = ":molybdenum.libera.chat CAP jwodder ACK :sasl";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        let outgoing = cmd
            .get_client_messages()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["AUTHENTICATE :PLAIN"]);
        assert!(!cmd.is_done());

        let m = "AUTHENTICATE +";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        let outgoing = cmd
            .get_client_messages()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["AUTHENTICATE :andvZGRlcgBqd29kZGVyAGh1bnRlcjI="]);
        assert!(!cmd.is_done());

        let m = ":molybdenum.libera.chat 900 jwodder jwodder!jwuser@127.0.0.1 jwodder :You are now logged in as jwodder";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        assert!(cmd.get_client_messages().is_empty());
        assert!(!cmd.is_done());

        let m = ":molybdenum.libera.chat 903 jwodder :SASL authentication successful";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        let outgoing = cmd
            .get_client_messages()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["CAP END"]);
        assert!(!cmd.is_done());

        let incoming = [
            ":molybdenum.libera.chat 001 jwodder :Welcome to the Libera.Chat Internet Relay Chat Network jwodder",
            ":molybdenum.libera.chat 002 jwodder :Your host is molybdenum.libera.chat[2607:5300:205:300::ae0/6697], running version solanum-1.0-dev",
            ":molybdenum.libera.chat 003 jwodder :This server was created Thu Jul 18 2024 at 16:57:02 UTC",
            ":molybdenum.libera.chat 004 jwodder molybdenum.libera.chat solanum-1.0-dev DGIMQRSZaghilopsuwz CFILMPQRSTbcefgijklmnopqrstuvz bkloveqjfI",
            ":molybdenum.libera.chat 005 jwodder ACCOUNTEXTBAN=a WHOX KNOCK MONITOR=100 ETRACE FNC SAFELIST ELIST=CMNTU CALLERID=g CHANTYPES=# EXCEPTS INVEX :are supported by this server",
            ":molybdenum.libera.chat 005 jwodder CHANMODES=eIbq,k,flj,CFLMPQRSTcgimnprstuz CHANLIMIT=#:250 PREFIX=(ov)@+ MAXLIST=bqeI:100 MODES=4 NETWORK=Libera.Chat STATUSMSG=@+ CASEMAPPING=rfc1459 NICKLEN=16 MAXNICKLEN=16 CHANNELLEN=50 TOPICLEN=390 :are supported by this server",
            ":molybdenum.libera.chat 005 jwodder DEAF=D TARGMAX=NAMES:1,LIST:1,KICK:1,WHOIS:1,PRIVMSG:4,NOTICE:4,ACCEPT:,MONITOR: EXTBAN=$,agjrxz :are supported by this server",
            ":molybdenum.libera.chat 251 jwodder :There are 62 users and 31502 invisible on 28 servers",
            ":molybdenum.libera.chat 252 jwodder 40 :IRC Operators online",
            ":molybdenum.libera.chat 253 jwodder 66 :unknown connection(s)",
            ":molybdenum.libera.chat 254 jwodder 22798 :channels formed",
            ":molybdenum.libera.chat 255 jwodder :I have 2700 clients and 1 servers",
            ":molybdenum.libera.chat 265 jwodder 2700 3071 :Current local users 2700, max 3071",
            ":molybdenum.libera.chat 266 jwodder 31564 34153 :Current global users 31564, max 34153",
            ":molybdenum.libera.chat 250 jwodder :Highest connection count: 3072 (3071 clients) (781421 connections received)",
            ":molybdenum.libera.chat 375 jwodder :- molybdenum.libera.chat Message of the Day - ",
            ":molybdenum.libera.chat 372 jwodder :- -[ Molybdenum ]-[ Atomic number: 42 ]-[ Chemical symbol: Mo ]-",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Molybdenum has been known throughout history, though its",
            ":molybdenum.libera.chat 372 jwodder :- properties meant that it was confused with lead ores. It was",
            ":molybdenum.libera.chat 372 jwodder :- first identified as a distinct element in 1778, meaning our",
            ":molybdenum.libera.chat 372 jwodder :- modern understanding of its existence reaches about as far",
            ":molybdenum.libera.chat 372 jwodder :- back as American independence.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Molybdenum has a very high melting point, 2623°C, the fifth-",
            ":molybdenum.libera.chat 372 jwodder :- highest temperature of any element.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- While pure molybdenum is only about as strong as tooth enamel,",
            ":molybdenum.libera.chat 372 jwodder :- it forms strong alloys that are resistant to corrosion and",
            ":molybdenum.libera.chat 372 jwodder :- stable under varying temperatures. As a result, these alloys",
            ":molybdenum.libera.chat 372 jwodder :- are used for metal armour plates, high-performance aircraft",
            ":molybdenum.libera.chat 372 jwodder :- parts, and pumps for molten metal flows.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- In its elemental form, it can be used as a fertiliser, and the",
            ":molybdenum.libera.chat 372 jwodder :- isotope molybdenum-99 is used as a source of technetium-99m,",
            ":molybdenum.libera.chat 372 jwodder :- an important medical radioisotope used for 3D bone scans and",
            ":molybdenum.libera.chat 372 jwodder :- the investigation of blood flow when diagnosing heart disease.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Welcome to Libera Chat, the IRC network for",
            ":molybdenum.libera.chat 372 jwodder :- free & open-source software and peer directed projects.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Use of Libera Chat is governed by our network policies.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- To reduce network abuses we perform open proxy checks",
            ":molybdenum.libera.chat 372 jwodder :- on hosts at connection time.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Please visit us in #libera for questions and support.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Website and documentation:  https://libera.chat",
            ":molybdenum.libera.chat 372 jwodder :- Webchat:                    https://web.libera.chat",
            ":molybdenum.libera.chat 372 jwodder :- Network policies:           https://libera.chat/policies",
            ":molybdenum.libera.chat 372 jwodder :- Email:                      support@libera.chat",
            ":molybdenum.libera.chat 376 jwodder :End of /MOTD command.",
        ];
        for m in incoming {
            let msg = m.parse::<Message>().unwrap();
            assert!(cmd.handle_message(&msg));
            assert!(cmd.get_client_messages().is_empty());
            assert!(!cmd.is_done());
        }

        let m = ":jwodder MODE jwodder :+Ziw";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        assert!(cmd.get_client_messages().is_empty());
        assert!(cmd.is_done());

        let output = cmd.get_output().unwrap();
        pretty_assertions::assert_eq!(
            output,
            LoginOutput {
                capabilities: Some(vec![
                    ("account-notify".parse::<Capability>().unwrap(), None),
                    ("away-notify".parse::<Capability>().unwrap(), None),
                    ("chghost".parse::<Capability>().unwrap(), None),
                    ("extended-join".parse::<Capability>().unwrap(), None),
                    ("multi-prefix".parse::<Capability>().unwrap(), None),
                    (
                        "sasl".parse::<Capability>().unwrap(),
                        Some(
                            "ECDSA-NIST256P-CHALLENGE,EXTERNAL,PLAIN,SCRAM-SHA-512"
                                .parse::<CapabilityValue>()
                                .unwrap()
                        )
                    ),
                    ("tls".parse::<Capability>().unwrap(), None),
                    ("account-tag".parse::<Capability>().unwrap(), None),
                    ("cap-notify".parse::<Capability>().unwrap(), None),
                    ("echo-message".parse::<Capability>().unwrap(), None),
                    ("server-time".parse::<Capability>().unwrap(), None),
                    (
                        "solanum.chat/identify-msg".parse::<Capability>().unwrap(),
                        None
                    ),
                    ("solanum.chat/oper".parse::<Capability>().unwrap(), None),
                    ("solanum.chat/realhost".parse::<Capability>().unwrap(), None),
                ]),
                my_nick: "jwodder".parse::<Nickname>().unwrap(),
                welcome_msg: "Welcome to the Libera.Chat Internet Relay Chat Network jwodder".into(),
                yourhost_msg: "Your host is molybdenum.libera.chat[2607:5300:205:300::ae0/6697], running version solanum-1.0-dev".into(),
                created_msg: "This server was created Thu Jul 18 2024 at 16:57:02 UTC".into(),
                server_info: ServerInfo {
                    name: "molybdenum.libera.chat".into(),
                    version: "solanum-1.0-dev".into(),
                    user_modes: "DGIMQRSZaghilopsuwz".into(),
                    channel_modes: "CFILMPQRSTbcefgijklmnopqrstuvz".into(),
                    param_channel_modes: Some("bkloveqjfI".to_owned()),
                },
                isupport: [
                    "ACCOUNTEXTBAN=a",
                    "WHOX",
                    "KNOCK",
                    "MONITOR=100",
                    "ETRACE",
                    "FNC",
                    "SAFELIST",
                    "ELIST=CMNTU",
                    "CALLERID=g",
                    "CHANTYPES=#",
                    "EXCEPTS",
                    "INVEX",
                    "CHANMODES=eIbq,k,flj,CFLMPQRSTcgimnprstuz",
                    "CHANLIMIT=#:250",
                    "PREFIX=(ov)@+",
                    "MAXLIST=bqeI:100",
                    "MODES=4",
                    "NETWORK=Libera.Chat",
                    "STATUSMSG=@+",
                    "CASEMAPPING=rfc1459",
                    "NICKLEN=16",
                    "MAXNICKLEN=16",
                    "CHANNELLEN=50",
                    "TOPICLEN=390",
                    "DEAF=D",
                    "TARGMAX=NAMES:1,LIST:1,KICK:1,WHOIS:1,PRIVMSG:4,NOTICE:4,ACCEPT:,MONITOR:",
                    "EXTBAN=$,agjrxz",
                ]
                .into_iter()
                .map(str::parse::<ISupportParam>)
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
                luser_stats: LuserStats {
                    operators: Some(40),
                    unknown_connections: Some(66),
                    channels: Some(22798),
                    local_clients: Some(2700),
                    max_local_clients: Some(3071),
                    global_clients: Some(31564),
                    max_global_clients: Some(34153),
                },
                motd: Some(
                    concat!(
                        "- molybdenum.libera.chat Message of the Day - \n",
                        "- -[ Molybdenum ]-[ Atomic number: 42 ]-[ Chemical symbol: Mo ]-\n",
                        "-  \n",
                        "- Molybdenum has been known throughout history, though its\n",
                        "- properties meant that it was confused with lead ores. It was\n",
                        "- first identified as a distinct element in 1778, meaning our\n",
                        "- modern understanding of its existence reaches about as far\n",
                        "- back as American independence.\n",
                        "-  \n",
                        "- Molybdenum has a very high melting point, 2623°C, the fifth-\n",
                        "- highest temperature of any element.\n",
                        "-  \n",
                        "- While pure molybdenum is only about as strong as tooth enamel,\n",
                        "- it forms strong alloys that are resistant to corrosion and\n",
                        "- stable under varying temperatures. As a result, these alloys\n",
                        "- are used for metal armour plates, high-performance aircraft\n",
                        "- parts, and pumps for molten metal flows.\n",
                        "-  \n",
                        "- In its elemental form, it can be used as a fertiliser, and the\n",
                        "- isotope molybdenum-99 is used as a source of technetium-99m,\n",
                        "- an important medical radioisotope used for 3D bone scans and\n",
                        "- the investigation of blood flow when diagnosing heart disease.\n",
                        "-  \n",
                        "-  \n",
                        "- Welcome to Libera Chat, the IRC network for\n",
                        "- free & open-source software and peer directed projects.\n",
                        "-  \n",
                        "- Use of Libera Chat is governed by our network policies.\n",
                        "-  \n",
                        "- To reduce network abuses we perform open proxy checks\n",
                        "- on hosts at connection time.\n",
                        "-  \n",
                        "- Please visit us in #libera for questions and support.\n",
                        "-  \n",
                        "- Website and documentation:  https://libera.chat\n",
                        "- Webchat:                    https://web.libera.chat\n",
                        "- Network policies:           https://libera.chat/policies\n",
                        "- Email:                      support@libera.chat\n",
                        "End of /MOTD command.",
                    )
                    .to_owned()
                ),
                mode: Some("+Ziw".parse::<ModeString>().unwrap()),
            }
        );
    }

    #[test]
    fn sasl_client_non_sasl_server() {
        let params = LoginParams {
            password: "hunter2".parse::<TrailingParam>().unwrap(),
            nickname: "jwodder".parse::<Nickname>().unwrap(),
            username: "jwuser".parse::<Username>().unwrap(),
            realname: "Just this guy, you know?".parse::<TrailingParam>().unwrap(),
            sasl: true,
        };
        let mut cmd = Login::new(params);

        let outgoing = cmd
            .get_client_messages()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(
            outgoing,
            [
                "CAP LS 302",
                "PASS :hunter2",
                "NICK jwodder",
                "USER jwuser 0 * :Just this guy, you know?"
            ]
        );

        let notices = [
            ":molybdenum.libera.chat NOTICE * :*** Checking Ident",
            ":molybdenum.libera.chat NOTICE * :*** Looking up your hostname...",
            ":molybdenum.libera.chat NOTICE * :*** Couldn't look up your hostname",
            ":molybdenum.libera.chat NOTICE * :*** No Ident response",
        ];
        for m in notices {
            let msg = m.parse::<Message>().unwrap();
            assert!(!cmd.handle_message(&msg));
            assert!(cmd.get_client_messages().is_empty());
            assert!(!cmd.is_done());
        }

        let m = ":molybdenum.libera.chat 421 jwodder CAP :What's a CAP?";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        assert!(cmd.get_client_messages().is_empty());
        assert!(!cmd.is_done());

        let incoming = [
            ":molybdenum.libera.chat 001 jwodder :Welcome to the Libera.Chat Internet Relay Chat Network jwodder",
            ":molybdenum.libera.chat 002 jwodder :Your host is molybdenum.libera.chat[2607:5300:205:300::ae0/6697], running version solanum-1.0-dev",
            ":molybdenum.libera.chat 003 jwodder :This server was created Thu Jul 18 2024 at 16:57:02 UTC",
            ":molybdenum.libera.chat 004 jwodder molybdenum.libera.chat solanum-1.0-dev DGIMQRSZaghilopsuwz CFILMPQRSTbcefgijklmnopqrstuvz bkloveqjfI",
            ":molybdenum.libera.chat 005 jwodder ACCOUNTEXTBAN=a WHOX KNOCK MONITOR=100 ETRACE FNC SAFELIST ELIST=CMNTU CALLERID=g CHANTYPES=# EXCEPTS INVEX :are supported by this server",
            ":molybdenum.libera.chat 005 jwodder CHANMODES=eIbq,k,flj,CFLMPQRSTcgimnprstuz CHANLIMIT=#:250 PREFIX=(ov)@+ MAXLIST=bqeI:100 MODES=4 NETWORK=Libera.Chat STATUSMSG=@+ CASEMAPPING=rfc1459 NICKLEN=16 MAXNICKLEN=16 CHANNELLEN=50 TOPICLEN=390 :are supported by this server",
            ":molybdenum.libera.chat 005 jwodder DEAF=D TARGMAX=NAMES:1,LIST:1,KICK:1,WHOIS:1,PRIVMSG:4,NOTICE:4,ACCEPT:,MONITOR: EXTBAN=$,agjrxz :are supported by this server",
            ":molybdenum.libera.chat 251 jwodder :There are 62 users and 31502 invisible on 28 servers",
            ":molybdenum.libera.chat 252 jwodder 40 :IRC Operators online",
            ":molybdenum.libera.chat 253 jwodder 66 :unknown connection(s)",
            ":molybdenum.libera.chat 254 jwodder 22798 :channels formed",
            ":molybdenum.libera.chat 255 jwodder :I have 2700 clients and 1 servers",
            ":molybdenum.libera.chat 265 jwodder 2700 3071 :Current local users 2700, max 3071",
            ":molybdenum.libera.chat 266 jwodder 31564 34153 :Current global users 31564, max 34153",
            ":molybdenum.libera.chat 250 jwodder :Highest connection count: 3072 (3071 clients) (781421 connections received)",
            ":molybdenum.libera.chat 375 jwodder :- molybdenum.libera.chat Message of the Day - ",
            ":molybdenum.libera.chat 372 jwodder :- -[ Molybdenum ]-[ Atomic number: 42 ]-[ Chemical symbol: Mo ]-",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Molybdenum has been known throughout history, though its",
            ":molybdenum.libera.chat 372 jwodder :- properties meant that it was confused with lead ores. It was",
            ":molybdenum.libera.chat 372 jwodder :- first identified as a distinct element in 1778, meaning our",
            ":molybdenum.libera.chat 372 jwodder :- modern understanding of its existence reaches about as far",
            ":molybdenum.libera.chat 372 jwodder :- back as American independence.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Molybdenum has a very high melting point, 2623°C, the fifth-",
            ":molybdenum.libera.chat 372 jwodder :- highest temperature of any element.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- While pure molybdenum is only about as strong as tooth enamel,",
            ":molybdenum.libera.chat 372 jwodder :- it forms strong alloys that are resistant to corrosion and",
            ":molybdenum.libera.chat 372 jwodder :- stable under varying temperatures. As a result, these alloys",
            ":molybdenum.libera.chat 372 jwodder :- are used for metal armour plates, high-performance aircraft",
            ":molybdenum.libera.chat 372 jwodder :- parts, and pumps for molten metal flows.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- In its elemental form, it can be used as a fertiliser, and the",
            ":molybdenum.libera.chat 372 jwodder :- isotope molybdenum-99 is used as a source of technetium-99m,",
            ":molybdenum.libera.chat 372 jwodder :- an important medical radioisotope used for 3D bone scans and",
            ":molybdenum.libera.chat 372 jwodder :- the investigation of blood flow when diagnosing heart disease.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Welcome to Libera Chat, the IRC network for",
            ":molybdenum.libera.chat 372 jwodder :- free & open-source software and peer directed projects.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Use of Libera Chat is governed by our network policies.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- To reduce network abuses we perform open proxy checks",
            ":molybdenum.libera.chat 372 jwodder :- on hosts at connection time.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Please visit us in #libera for questions and support.",
            ":molybdenum.libera.chat 372 jwodder :-  ",
            ":molybdenum.libera.chat 372 jwodder :- Website and documentation:  https://libera.chat",
            ":molybdenum.libera.chat 372 jwodder :- Webchat:                    https://web.libera.chat",
            ":molybdenum.libera.chat 372 jwodder :- Network policies:           https://libera.chat/policies",
            ":molybdenum.libera.chat 372 jwodder :- Email:                      support@libera.chat",
            ":molybdenum.libera.chat 376 jwodder :End of /MOTD command.",
        ];
        for m in incoming {
            let msg = m.parse::<Message>().unwrap();
            assert!(cmd.handle_message(&msg));
            assert!(cmd.get_client_messages().is_empty());
            assert!(!cmd.is_done());
        }

        let m = ":jwodder MODE jwodder :+Ziw";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        assert!(cmd.get_client_messages().is_empty());
        assert!(cmd.is_done());

        let output = cmd.get_output().unwrap();
        pretty_assertions::assert_eq!(
            output,
            LoginOutput {
                capabilities: None,
                my_nick: "jwodder".parse::<Nickname>().unwrap(),
                welcome_msg: "Welcome to the Libera.Chat Internet Relay Chat Network jwodder".into(),
                yourhost_msg: "Your host is molybdenum.libera.chat[2607:5300:205:300::ae0/6697], running version solanum-1.0-dev".into(),
                created_msg: "This server was created Thu Jul 18 2024 at 16:57:02 UTC".into(),
                server_info: ServerInfo {
                    name: "molybdenum.libera.chat".into(),
                    version: "solanum-1.0-dev".into(),
                    user_modes: "DGIMQRSZaghilopsuwz".into(),
                    channel_modes: "CFILMPQRSTbcefgijklmnopqrstuvz".into(),
                    param_channel_modes: Some("bkloveqjfI".to_owned()),
                },
                isupport: [
                    "ACCOUNTEXTBAN=a",
                    "WHOX",
                    "KNOCK",
                    "MONITOR=100",
                    "ETRACE",
                    "FNC",
                    "SAFELIST",
                    "ELIST=CMNTU",
                    "CALLERID=g",
                    "CHANTYPES=#",
                    "EXCEPTS",
                    "INVEX",
                    "CHANMODES=eIbq,k,flj,CFLMPQRSTcgimnprstuz",
                    "CHANLIMIT=#:250",
                    "PREFIX=(ov)@+",
                    "MAXLIST=bqeI:100",
                    "MODES=4",
                    "NETWORK=Libera.Chat",
                    "STATUSMSG=@+",
                    "CASEMAPPING=rfc1459",
                    "NICKLEN=16",
                    "MAXNICKLEN=16",
                    "CHANNELLEN=50",
                    "TOPICLEN=390",
                    "DEAF=D",
                    "TARGMAX=NAMES:1,LIST:1,KICK:1,WHOIS:1,PRIVMSG:4,NOTICE:4,ACCEPT:,MONITOR:",
                    "EXTBAN=$,agjrxz",
                ]
                .into_iter()
                .map(str::parse::<ISupportParam>)
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
                luser_stats: LuserStats {
                    operators: Some(40),
                    unknown_connections: Some(66),
                    channels: Some(22798),
                    local_clients: Some(2700),
                    max_local_clients: Some(3071),
                    global_clients: Some(31564),
                    max_global_clients: Some(34153),
                },
                motd: Some(
                    concat!(
                        "- molybdenum.libera.chat Message of the Day - \n",
                        "- -[ Molybdenum ]-[ Atomic number: 42 ]-[ Chemical symbol: Mo ]-\n",
                        "-  \n",
                        "- Molybdenum has been known throughout history, though its\n",
                        "- properties meant that it was confused with lead ores. It was\n",
                        "- first identified as a distinct element in 1778, meaning our\n",
                        "- modern understanding of its existence reaches about as far\n",
                        "- back as American independence.\n",
                        "-  \n",
                        "- Molybdenum has a very high melting point, 2623°C, the fifth-\n",
                        "- highest temperature of any element.\n",
                        "-  \n",
                        "- While pure molybdenum is only about as strong as tooth enamel,\n",
                        "- it forms strong alloys that are resistant to corrosion and\n",
                        "- stable under varying temperatures. As a result, these alloys\n",
                        "- are used for metal armour plates, high-performance aircraft\n",
                        "- parts, and pumps for molten metal flows.\n",
                        "-  \n",
                        "- In its elemental form, it can be used as a fertiliser, and the\n",
                        "- isotope molybdenum-99 is used as a source of technetium-99m,\n",
                        "- an important medical radioisotope used for 3D bone scans and\n",
                        "- the investigation of blood flow when diagnosing heart disease.\n",
                        "-  \n",
                        "-  \n",
                        "- Welcome to Libera Chat, the IRC network for\n",
                        "- free & open-source software and peer directed projects.\n",
                        "-  \n",
                        "- Use of Libera Chat is governed by our network policies.\n",
                        "-  \n",
                        "- To reduce network abuses we perform open proxy checks\n",
                        "- on hosts at connection time.\n",
                        "-  \n",
                        "- Please visit us in #libera for questions and support.\n",
                        "-  \n",
                        "- Website and documentation:  https://libera.chat\n",
                        "- Webchat:                    https://web.libera.chat\n",
                        "- Network policies:           https://libera.chat/policies\n",
                        "- Email:                      support@libera.chat\n",
                        "End of /MOTD command.",
                    )
                    .to_owned()
                ),
                mode: Some("+Ziw".parse::<ModeString>().unwrap()),
            }
        );
    }
}
