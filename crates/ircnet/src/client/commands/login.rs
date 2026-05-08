use super::Command;
use crate::sasl::{SaslError, SaslFlow, SaslMachine, SaslMechanism};
use irctext::{
    ClientMessage, ClientMessageParts, Message, Payload, Reply, ReplyParts, TrailingParam,
    TryFromStringError,
    clientmsgs::{
        Authenticate, Cap, CapEnd, CapLsRequest, CapReq, Capability, CapabilityRequest,
        CapabilityValue, Mode, Nick, Pass, User,
    },
    types::{
        CaseMapping, ISupportParam, ISupportSetting, ModeString, Nickname, ParseCaseMappingError,
        ReplyTarget, Username,
    },
};
use mitsein::vec1::Vec1;
use replace_with::{replace_with, replace_with_and_return};
use std::collections::{BTreeMap, HashSet, VecDeque};
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
    #[cfg_attr(
        feature = "serde",
        serde(rename = "sasl-mechanisms", default = "default_sasl_mechs")
    )]
    pub sasl_mechanisms: Vec1<SaslMechanism>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub capabilities: BTreeMap<Capability, CapDesire>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum CapDesire {
    Required,
    Optional,
}

#[cfg(feature = "serde")]
fn default_sasl() -> bool {
    true
}

#[cfg(feature = "serde")]
fn default_sasl_mechs() -> Vec1<SaslMechanism> {
    Vec1::from([
        SaslMechanism::ScramSha512,
        SaslMechanism::ScramSha256,
        SaslMechanism::ScramSha1,
        SaslMechanism::Plain,
    ])
}

#[derive(Debug)]
pub struct Login {
    outgoing: Vec<ClientMessage>,
    state: State,
}

impl Login {
    pub fn new(params: LoginParams) -> Login {
        let use_cap = params.sasl || !params.capabilities.is_empty();
        let mut outgoing = Vec::with_capacity(4);
        if use_cap {
            outgoing.push(ClientMessage::from(CapLsRequest::new_with_version(302)));
        }
        outgoing.push(ClientMessage::from(Pass::new(params.password.clone())));
        outgoing.push(ClientMessage::from(Nick::new(params.nickname.clone())));
        outgoing.push(ClientMessage::from(User::new(
            params.username,
            params.realname,
        )));
        let sasl = params.sasl.then(|| SaslBuilder {
            mechanisms: VecDeque::from_iter(params.sasl_mechanisms),
            nickname: params.nickname,
            password: params.password,
        });
        Login {
            outgoing,
            state: if use_cap {
                State::Start {
                    caps_desired: params.capabilities,
                    sasl,
                }
            } else {
                State::Awaiting001 {
                    capabilities: None,
                    capabilities_enabled: HashSet::new(),
                }
            },
        }
    }
}

// Order of replies on successful login:
// - With capabilities:
//     - S: `CAP * LS :…`, possibly preceded by zero or more `CAP * LS * :…`
//          - If no supported mechanisms are in sasl, send CAP END and await
//            RPL_WELCOME
//     - For each capability:
//         - C: CAP REQ {cap}
//         - S: CAP * ACK {cap}
//             - Or `CAP * NAK {cap}`, in which case we error out
//     - If SASL:
//         - C: AUTHENTICATE <mechanism>
//         - S: AUTHENTICATE +
//             - Or ERR_SASLFAIL (904) or RPL_SASLMECHS (908)
//         - SASL flow
//         - S: RPL_LOGGEDIN (900)
//         - S: RPL_SASLSUCCESS (903)
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
                    if let State::Start { caps_desired, .. } = &self.state
                        && let Reply::UnknownCommand(r) = rpl
                        && r.command() == "CAP"
                    {
                        if caps_desired.values().any(|&d| d == CapDesire::Required) {
                            self.state = State::error(LoginError::CapNotSupported);
                        } else {
                            self.state = State::Awaiting001 {
                                capabilities: None,
                                capabilities_enabled: HashSet::new(),
                            };
                        }
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
                        Reply::SaslFail(r)
                            if let State::SentMechanism {
                                sasl, sasl_flow, ..
                            } = &mut self.state =>
                        {
                            match sasl.start_next_flow() {
                                Ok(Some((msg, new_flow))) => {
                                    self.outgoing.push(msg);
                                    *sasl_flow = new_flow;
                                    return true;
                                }
                                Ok(None) => LoginError::SaslFail {
                                    message: r.message().to_string(),
                                },
                                Err(e) => e,
                            }
                        }
                        Reply::SaslAlready(r) => LoginError::SaslAlready {
                            message: r.message().to_string(),
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
                    let (b, msgs) = self.state.handle_auth(auth);
                    self.outgoing.extend(msgs.into_iter().map(Into::into));
                    b
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

#[derive(Debug)]
enum State {
    Start {
        caps_desired: BTreeMap<Capability, CapDesire>,
        sasl: Option<SaslBuilder>,
    },
    ListingCaps {
        capabilities: Vec<(Capability, Option<CapabilityValue>)>,
        caps_desired: BTreeMap<Capability, CapDesire>,
        sasl: Option<SaslBuilder>,
    },
    AwaitingAck {
        capabilities: Vec<(Capability, Option<CapabilityValue>)>,
        capabilities_enabled: HashSet<Capability>,
        caps_to_enable: VecDeque<Capability>,
        for_cap: Capability,
        sasl: Option<SaslBuilder>,
    },
    SentMechanism {
        capabilities: Vec<(Capability, Option<CapabilityValue>)>,
        capabilities_enabled: HashSet<Capability>,
        sasl: SaslBuilder,
        sasl_flow: SaslMachine,
    },
    Sasl {
        capabilities: Vec<(Capability, Option<CapabilityValue>)>,
        capabilities_enabled: HashSet<Capability>,
        sasl_flow: SaslMachine,
    },
    SaslDone {
        capabilities: Vec<(Capability, Option<CapabilityValue>)>,
        capabilities_enabled: HashSet<Capability>,
    },
    Got900 {
        capabilities: Vec<(Capability, Option<CapabilityValue>)>,
        capabilities_enabled: HashSet<Capability>,
    },
    Awaiting001 {
        capabilities: Option<Vec<(Capability, Option<CapabilityValue>)>>,
        capabilities_enabled: HashSet<Capability>,
    },
    Got001 {
        my_nick: Nickname,
        welcome_msg: String,
        capabilities: Option<Vec<(Capability, Option<CapabilityValue>)>>,
        capabilities_enabled: HashSet<Capability>,
    },
    Got002 {
        my_nick: Nickname,
        welcome_msg: String,
        yourhost_msg: String,
        capabilities: Option<Vec<(Capability, Option<CapabilityValue>)>>,
        capabilities_enabled: HashSet<Capability>,
    },
    Got003 {
        my_nick: Nickname,
        welcome_msg: String,
        yourhost_msg: String,
        created_msg: String,
        capabilities: Option<Vec<(Capability, Option<CapabilityValue>)>>,
        capabilities_enabled: HashSet<Capability>,
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
                (State::Start { caps_desired, .. }, Reply::Welcome(r)) => {
                    if caps_desired.values().any(|&d| d == CapDesire::Required) {
                        ((true, None), State::error(LoginError::CapNotSupported))
                    } else if let ReplyTarget::Nick(nick) = r.client() {
                        let my_nick = nick.clone();
                        (
                            (true, None),
                            State::Got001 {
                                my_nick,
                                welcome_msg: r.message().to_owned(),
                                capabilities: None,
                                capabilities_enabled: HashSet::new(),
                            },
                        )
                    } else {
                        ((true, None), State::error(LoginError::StarWelcome))
                    }
                }
                (
                    State::SentMechanism {
                        capabilities,
                        capabilities_enabled,
                        mut sasl,
                        ..
                    },
                    Reply::SaslMechs(r),
                ) => {
                    let server_mechs = r
                        .mechanisms()
                        .split(',')
                        .filter_map(|m| m.parse::<SaslMechanism>().ok())
                        .collect::<HashSet<_>>();
                    sasl.restrict_mechanisms(server_mechs);
                    match sasl.start_next_flow() {
                        Ok(Some((msg, sasl_flow))) => (
                            (true, Some(msg)),
                            State::SentMechanism {
                                capabilities,
                                capabilities_enabled,
                                sasl,
                                sasl_flow,
                            },
                        ),
                        Ok(None) => {
                            tracing::debug!(
                                "Server does not support any enabled SASL mechanisms; skipping SASL"
                            );
                            (
                                (true, Some(ClientMessage::from(CapEnd))),
                                State::Awaiting001 {
                                    capabilities: Some(capabilities),
                                    capabilities_enabled,
                                },
                            )
                        }
                        Err(e) => ((true, None), State::error(e)),
                    }
                }
                (
                    State::SaslDone {
                        capabilities,
                        capabilities_enabled,
                    },
                    Reply::LoggedIn(_),
                ) => (
                    (true, None),
                    State::Got900 {
                        capabilities,
                        capabilities_enabled,
                    },
                ),
                (
                    State::Got900 {
                        capabilities,
                        capabilities_enabled,
                    },
                    Reply::SaslSuccess(_),
                ) => (
                    (true, Some(ClientMessage::from(CapEnd))),
                    State::Awaiting001 {
                        capabilities: Some(capabilities),
                        capabilities_enabled,
                    },
                ),
                (
                    State::Awaiting001 {
                        capabilities,
                        capabilities_enabled,
                    },
                    Reply::Welcome(r),
                ) => {
                    if let ReplyTarget::Nick(nick) = r.client() {
                        let my_nick = nick.clone();
                        (
                            (true, None),
                            State::Got001 {
                                my_nick,
                                welcome_msg: r.message().to_owned(),
                                capabilities,
                                capabilities_enabled,
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
                        capabilities_enabled,
                    },
                    Reply::YourHost(r),
                ) => (
                    (true, None),
                    State::Got002 {
                        my_nick,
                        welcome_msg,
                        yourhost_msg: r.message().to_owned(),
                        capabilities,
                        capabilities_enabled,
                    },
                ),
                (
                    State::Got002 {
                        my_nick,
                        welcome_msg,
                        yourhost_msg,
                        capabilities,
                        capabilities_enabled,
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
                        capabilities_enabled,
                    },
                ),
                (
                    State::Got003 {
                        my_nick,
                        welcome_msg,
                        yourhost_msg,
                        created_msg,
                        capabilities,
                        capabilities_enabled,
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
                        capabilities_enabled,
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
                (State::Got005(mut output) | State::Lusers(mut output), Reply::StatsConn(r)) => {
                    output.luser_stats.statsconn_msg = Some(r.message().to_owned());
                    ((true, None), State::Lusers(output))
                }
                (State::Got005(mut output) | State::Lusers(mut output), Reply::LuserClient(r)) => {
                    output.luser_stats.luserclient_msg = Some(r.message().to_owned());
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
                (State::Got005(mut output) | State::Lusers(mut output), Reply::LuserMe(r)) => {
                    output.luser_stats.luserme_msg = Some(r.message().to_owned());
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
                (State::Got005(output) | State::Lusers(output), Reply::NoMotd(_)) => (
                    (true, None),
                    State::AwaitingMode {
                        output,
                        timeout: Some(MODE_TIMEOUT),
                    },
                ),
                (st @ State::Got005(_), _) => ((false, None), st), // Accept "other numerics and messages" after RPL_ISUPPORT
                (State::Motd(mut output), Reply::Motd(r)) => {
                    if let Some(s) = output.motd.as_mut() {
                        s.push('\n');
                        s.push_str(r.message());
                    }
                    ((true, None), State::Motd(output))
                }
                (State::Motd(output), Reply::EndOfMotd(_)) => (
                    (true, None),
                    State::AwaitingMode {
                        output,
                        timeout: Some(MODE_TIMEOUT),
                    },
                ),
                (State::AwaitingMode { mut output, .. }, Reply::UModeIs(r)) => {
                    output.mode = Some(r.user_modes().clone());
                    ((true, None), State::Done(Some(Ok(output))))
                }
                (State::AwaitingMode { output, .. }, _) => {
                    // If we get an unexpected reply while waiting for
                    // MODE/RPL_UMODEIS, consider login done.
                    ((false, None), State::Done(Some(Ok(output))))
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
                (State::Start { caps_desired, sasl }, Cap::LsResponse(m)) => {
                    let capabilities = m.capabilities.clone();
                    if m.continued {
                        (
                            None,
                            State::ListingCaps {
                                capabilities,
                                caps_desired,
                                sasl,
                            },
                        )
                    } else {
                        post_cap_ls(capabilities, caps_desired, sasl)
                    }
                }
                (
                    State::ListingCaps {
                        mut capabilities,
                        caps_desired,
                        sasl,
                    },
                    Cap::LsResponse(m),
                ) => {
                    capabilities.extend(m.capabilities.clone());
                    if m.continued {
                        (
                            None,
                            State::ListingCaps {
                                capabilities,
                                caps_desired,
                                sasl,
                            },
                        )
                    } else {
                        post_cap_ls(capabilities, caps_desired, sasl)
                    }
                }
                (
                    State::AwaitingAck {
                        capabilities,
                        mut capabilities_enabled,
                        mut caps_to_enable,
                        for_cap,
                        sasl,
                    },
                    Cap::Ack(m),
                ) => {
                    if m.capabilities.len() == 1
                        && m.capabilities[0].capability == for_cap
                        && !m.capabilities[0].disable
                    {
                        capabilities_enabled.insert(for_cap);
                        if let Some(for_cap) = caps_to_enable.pop_front() {
                            let cap_req = ClientMessage::from(CapReq {
                                capabilities: vec![CapabilityRequest::enable(for_cap.clone())],
                            });
                            (
                                Some(cap_req),
                                State::AwaitingAck {
                                    capabilities,
                                    caps_to_enable,
                                    capabilities_enabled,
                                    for_cap,
                                    sasl,
                                },
                            )
                        } else if let Some(mut sasl) = sasl {
                            sasl.restrict_mechanisms(servers_sasl_mechs(&capabilities));
                            match sasl.start_next_flow() {
                                Ok(Some((msg, sasl_flow))) => (
                                    Some(msg),
                                    State::SentMechanism {
                                        capabilities,
                                        capabilities_enabled,
                                        sasl,
                                        sasl_flow,
                                    },
                                ),
                                Ok(None) => (
                                    Some(ClientMessage::from(CapEnd)),
                                    State::Awaiting001 {
                                        capabilities: Some(capabilities),
                                        capabilities_enabled: HashSet::new(),
                                    },
                                ),
                                Err(e) => (None, State::error(e)),
                            }
                        } else {
                            (
                                None,
                                State::Awaiting001 {
                                    capabilities: Some(capabilities),
                                    capabilities_enabled,
                                },
                            )
                        }
                    } else {
                        (
                            None,
                            State::error(LoginError::BadAckNak {
                                requested: for_cap,
                                got: cap.to_irc_line(),
                            }),
                        )
                    }
                }
                (State::AwaitingAck { for_cap, .. }, Cap::Nak(m)) => {
                    if m.capabilities.len() == 1 && m.capabilities[0] == for_cap {
                        (
                            None,
                            State::error(LoginError::CapNakked {
                                capability: for_cap,
                            }),
                        )
                    } else {
                        (
                            None,
                            State::error(LoginError::BadAckNak {
                                requested: for_cap,
                                got: cap.to_irc_line(),
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

    fn handle_auth(&mut self, auth: &Authenticate) -> (bool, Vec<Authenticate>) {
        replace_with_and_return(
            self,
            || State::Void,
            |state| match state {
                State::SentMechanism {
                    capabilities,
                    capabilities_enabled,
                    mut sasl_flow,
                    ..
                }
                | State::Sasl {
                    capabilities,
                    capabilities_enabled,
                    mut sasl_flow,
                } => match sasl_flow.handle_message(auth) {
                    Ok(msgs) => {
                        if sasl_flow.is_done() {
                            (
                                (true, msgs),
                                State::SaslDone {
                                    capabilities,
                                    capabilities_enabled,
                                },
                            )
                        } else {
                            (
                                (true, msgs),
                                State::Sasl {
                                    capabilities,
                                    capabilities_enabled,
                                    sasl_flow,
                                },
                            )
                        }
                    }
                    Err(e) => ((true, Vec::new()), State::error(LoginError::Sasl(e))),
                },
                state => {
                    let expecting = state.expecting();
                    let msg = auth.to_irc_line();
                    (
                        (false, Vec::new()),
                        State::error(LoginError::Unexpected { expecting, msg }),
                    )
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
            State::Start { .. } => r#""CAP * LS" response or RPL_WELCOME (001) reply"#,
            State::ListingCaps { .. } => r#""CAP * LS" response"#,
            State::AwaitingAck { .. } => r#""CAP * ACK" response"#,
            State::SentMechanism { .. } => r#""AUTHENTICATE +" response"#,
            State::Sasl { .. } => r#""AUTHENTICATE" response"#,
            State::SaslDone { .. } => "RPL_LOGGEDIN (900) reply",
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

    /// The set of all capabilities enabled during login
    pub capabilities_enabled: HashSet<Capability>,

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

impl LoginOutput {
    pub fn get_isupport_setting<S: AsRef<str>>(&self, key: S) -> Option<&ISupportSetting> {
        let key = key.as_ref();
        self.isupport
            .iter()
            .filter_map(|param| (param.key == key).then_some(&param.setting))
            .next_back()
    }

    pub fn casemapping(&self) -> Result<CaseMapping, TryFromStringError<ParseCaseMappingError>> {
        if let Some(ISupportSetting::Value(value)) = self.get_isupport_setting("CASEMAPPING") {
            CaseMapping::try_from(value.to_string())
        } else {
            Ok(CaseMapping::default())
        }
    }

    pub fn botmode(&self) -> Option<char> {
        if let Some(ISupportSetting::Value(value)) = self.get_isupport_setting("BOT") {
            value.parse::<char>().ok()
        } else {
            None
        }
    }
}

/// Details about the IRC server as given in the `RPL_MYINFO` reply
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
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

    /// The message given in the `RPL_LUSERCLIENT` reply, or `None` if the
    /// reply was not sent
    pub luserclient_msg: Option<String>,

    /// The message given in the `RPL_LUSERME` reply, or `None` if the reply
    /// was not sent
    pub luserme_msg: Option<String>,

    /// The message given in the `RPL_STATSCONN` reply, or `None` if the reply
    /// was not sent
    pub statsconn_msg: Option<String>,
}

#[derive(Debug, Error)]
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
    #[error("login failed because server rejected SASL mechanism: {message:?}")]
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
    #[error("login failed because server does not support capability negotiation")]
    CapNotSupported,
    #[error("login failed because server does not support required capability {capability}")]
    RequiredCapNotSupported { capability: Capability },
    #[error(
        r#"login failed because server responded to "CAP REQ {requested}" with inapplicable {got:?}"#
    )]
    BadAckNak { requested: Capability, got: String },
    #[error("login failed because server NAKed our request to enable {capability}")]
    CapNakked { capability: Capability },
    #[error("login failed due to SASL failing")]
    Sasl(SaslError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SaslBuilder {
    mechanisms: VecDeque<SaslMechanism>,
    nickname: Nickname,
    password: TrailingParam,
}

impl SaslBuilder {
    fn restrict_mechanisms(&mut self, new_mechs: HashSet<SaslMechanism>) {
        self.mechanisms.retain(|m| new_mechs.contains(m));
    }

    fn pop_next_mechanism(&mut self) -> Option<SaslMechanism> {
        self.mechanisms.pop_front()
    }

    fn start_next_flow(&mut self) -> Result<Option<(ClientMessage, SaslMachine)>, LoginError> {
        if let Some(mech) = self.pop_next_mechanism() {
            match mech.new_flow(&self.nickname, &self.password) {
                Ok((flow, msg1)) => {
                    tracing::debug!("Starting SASL authentication with {mech} mechanism");
                    Ok(Some((ClientMessage::from(msg1), flow)))
                }
                Err(e) => Err(LoginError::Sasl(e)),
            }
        } else {
            Ok(None)
        }
    }
}

fn post_cap_ls(
    capabilities: Vec<(Capability, Option<CapabilityValue>)>,
    mut caps_desired: BTreeMap<Capability, CapDesire>,
    mut sasl: Option<SaslBuilder>,
) -> (Option<ClientMessage>, State) {
    if let Some(mut ss) = sasl.take() {
        ss.restrict_mechanisms(servers_sasl_mechs(&capabilities));
        if ss.mechanisms.is_empty() {
            tracing::debug!("Server does not support any enabled SASL mechanisms; skipping SASL");
            caps_desired.remove("sasl");
        } else {
            sasl = Some(ss);
            let sasl_cap = "sasl"
                .parse::<Capability>()
                .expect(r#""sasl" should be a valid capability name"#);
            caps_desired.insert(sasl_cap, CapDesire::Optional);
        }
    } else {
        caps_desired.remove("sasl");
    }
    let mut caps_to_enable = VecDeque::new();
    for (cap, desire) in caps_desired {
        if capabilities.iter().any(|c| c.0 == cap) {
            caps_to_enable.push_back(cap);
        } else if desire == CapDesire::Required {
            return (
                None,
                State::error(LoginError::RequiredCapNotSupported { capability: cap }),
            );
        }
    }
    if let Some(for_cap) = caps_to_enable.pop_front() {
        let cap_req = ClientMessage::from(CapReq {
            capabilities: vec![CapabilityRequest::enable(for_cap.clone())],
        });
        (
            Some(cap_req),
            State::AwaitingAck {
                capabilities,
                caps_to_enable,
                capabilities_enabled: HashSet::new(),
                for_cap,
                sasl,
            },
        )
    } else {
        let cap_end = ClientMessage::from(CapEnd);
        (
            Some(cap_end),
            State::Awaiting001 {
                capabilities: Some(capabilities),
                capabilities_enabled: HashSet::new(),
            },
        )
    }
}

fn servers_sasl_mechs(caps: &[(Capability, Option<CapabilityValue>)]) -> HashSet<SaslMechanism> {
    match caps
        .iter()
        .filter_map(|(key, value)| (key == "sasl").then_some(value.as_ref()))
        .next_back()
    {
        Some(Some(value)) => value
            .split(',')
            .filter_map(|m| m.parse::<SaslMechanism>().ok())
            .collect(),
        Some(None) => {
            // Server does not support SASL v3.2 / Capability Negotation 302
            // and so isn't telling us what SASL mechanisms it supports.  Just
            // try everything we support.
            SaslMechanism::iter().collect()
        }
        None => HashSet::new(),
    }
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
            sasl_mechanisms: Vec1::from_one(SaslMechanism::Plain),
            capabilities: BTreeMap::new(),
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
                capabilities_enabled: HashSet::new(),
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
                    luserclient_msg: Some("There are 62 users and 31502 invisible on 28 servers".to_owned()),
                    luserme_msg: Some("I have 2700 clients and 1 servers".to_owned()),
                    statsconn_msg: Some("Highest connection count: 3072 (3071 clients) (781421 connections received)".to_owned()),
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
                        "- Email:                      support@libera.chat",
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
            sasl_mechanisms: Vec1::from_one(SaslMechanism::Plain),
            capabilities: BTreeMap::new(),
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
                capabilities_enabled: HashSet::from(["sasl".parse::<Capability>().unwrap()]),
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
                    luserclient_msg: Some("There are 62 users and 31502 invisible on 28 servers".to_owned()),
                    luserme_msg: Some("I have 2700 clients and 1 servers".to_owned()),
                    statsconn_msg: Some("Highest connection count: 3072 (3071 clients) (781421 connections received)".to_owned()),
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
                        "- Email:                      support@libera.chat",
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
            sasl_mechanisms: Vec1::from_one(SaslMechanism::Plain),
            capabilities: BTreeMap::new(),
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
                capabilities_enabled: HashSet::new(),
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
                    luserclient_msg: Some("There are 62 users and 31502 invisible on 28 servers".to_owned()),
                    luserme_msg: Some("I have 2700 clients and 1 servers".to_owned()),
                    statsconn_msg: Some("Highest connection count: 3072 (3071 clients) (781421 connections received)".to_owned()),
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
                        "- Email:                      support@libera.chat",
                    )
                    .to_owned()
                ),
                mode: Some("+Ziw".parse::<ModeString>().unwrap()),
            }
        );
    }

    #[test]
    fn sasl_v31_server_saslmechs_err() {
        let params = LoginParams {
            password: "hunter2".parse::<TrailingParam>().unwrap(),
            nickname: "jwodder".parse::<Nickname>().unwrap(),
            username: "jwuser".parse::<Username>().unwrap(),
            realname: "Just this guy, you know?".parse::<TrailingParam>().unwrap(),
            sasl: true,
            sasl_mechanisms: Vec1::from([SaslMechanism::ScramSha256, SaslMechanism::Plain]),
            capabilities: BTreeMap::new(),
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

        let m = ":irc.example.com CAP * LS :sasl";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        let outgoing = cmd
            .get_client_messages()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["CAP REQ :sasl"]);
        assert!(!cmd.is_done());

        let m = ":irc.example.com CAP jwodder ACK :sasl";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        let outgoing = cmd
            .get_client_messages()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["AUTHENTICATE :SCRAM-SHA-256"]);
        assert!(!cmd.is_done());

        let m = ":irc.example.com 908 jwodder SCRAM-SHA-512,PLAIN :are available SASL mechanism";
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

        let m = ":irc.example.com 900 jwodder jwodder!jwuser@127.0.0.1 jwodder :You are now logged in as jwodder";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        assert!(cmd.get_client_messages().is_empty());
        assert!(!cmd.is_done());

        let m = ":irc.example.com 903 jwodder :SASL authentication successful";
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
            ":irc.example.com 001 jwodder :Welcome to the Example Internet Relay Chat Network, jwodder",
            ":irc.example.com 002 jwodder :Your host is irc.example.com, running version solanum-1.0-dev",
            ":irc.example.com 003 jwodder :This server was created Thu Jul 18 2024 at 16:57:02 UTC",
            ":irc.example.com 004 jwodder irc.example.com solanum-1.0-dev DGIMQRSZaghilopsuwz CFILMPQRSTbcefgijklmnopqrstuvz bkloveqjfI",
            ":irc.example.com 005 jwodder ACCOUNTEXTBAN=a WHOX KNOCK MONITOR=100 ETRACE FNC SAFELIST ELIST=CMNTU CALLERID=g CHANTYPES=# EXCEPTS INVEX :are supported by this server",
            ":irc.example.com 005 jwodder CHANMODES=eIbq,k,flj,CFLMPQRSTcgimnprstuz CHANLIMIT=#:250 PREFIX=(ov)@+ MAXLIST=bqeI:100 MODES=4 NETWORK=Libera.Chat STATUSMSG=@+ CASEMAPPING=rfc1459 NICKLEN=16 MAXNICKLEN=16 CHANNELLEN=50 TOPICLEN=390 :are supported by this server",
            ":irc.example.com 005 jwodder DEAF=D TARGMAX=NAMES:1,LIST:1,KICK:1,WHOIS:1,PRIVMSG:4,NOTICE:4,ACCEPT:,MONITOR: EXTBAN=$,agjrxz :are supported by this server",
            ":irc.example.com 251 jwodder :There are 62 users and 31502 invisible on 28 servers",
            ":irc.example.com 252 jwodder 40 :IRC Operators online",
            ":irc.example.com 253 jwodder 66 :unknown connection(s)",
            ":irc.example.com 254 jwodder 22798 :channels formed",
            ":irc.example.com 255 jwodder :I have 2700 clients and 1 servers",
            ":irc.example.com 265 jwodder 2700 3071 :Current local users 2700, max 3071",
            ":irc.example.com 266 jwodder 31564 34153 :Current global users 31564, max 34153",
            ":irc.example.com 250 jwodder :Highest connection count: 3072 (3071 clients) (781421 connections received)",
            ":irc.example.com 422 jwodder :No message today",
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
                capabilities: Some(vec![("sasl".parse::<Capability>().unwrap(), None)]),
                capabilities_enabled: HashSet::from(["sasl".parse::<Capability>().unwrap()]),
                my_nick: "jwodder".parse::<Nickname>().unwrap(),
                welcome_msg: "Welcome to the Example Internet Relay Chat Network, jwodder".into(),
                yourhost_msg: "Your host is irc.example.com, running version solanum-1.0-dev".into(),
                created_msg: "This server was created Thu Jul 18 2024 at 16:57:02 UTC".into(),
                server_info: ServerInfo {
                    name: "irc.example.com".into(),
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
                    luserclient_msg: Some("There are 62 users and 31502 invisible on 28 servers".to_owned()),
                    luserme_msg: Some("I have 2700 clients and 1 servers".to_owned()),
                    statsconn_msg: Some("Highest connection count: 3072 (3071 clients) (781421 connections received)".to_owned()),
                },
                motd: None,
                mode: Some("+Ziw".parse::<ModeString>().unwrap()),
            }
        );
    }

    #[test]
    fn required_and_optional_caps_optional_not_supported() {
        let params = LoginParams {
            password: "hunter2".parse::<TrailingParam>().unwrap(),
            nickname: "jwodder".parse::<Nickname>().unwrap(),
            username: "jwuser".parse::<Username>().unwrap(),
            realname: "Just this guy, you know?".parse::<TrailingParam>().unwrap(),
            sasl: true,
            sasl_mechanisms: Vec1::from_one(SaslMechanism::Plain),
            capabilities: BTreeMap::from([
                (
                    "message-tags".parse::<Capability>().unwrap(),
                    CapDesire::Required,
                ),
                (
                    "server-time".parse::<Capability>().unwrap(),
                    CapDesire::Optional,
                ),
            ]),
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

        let m = ":irc.example.com CAP * LS :account-notify away-notify sasl=ECDSA-NIST256P-CHALLENGE,EXTERNAL,PLAIN,SCRAM-SHA-512 message-tags";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        let outgoing = cmd
            .get_client_messages()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["CAP REQ :message-tags"]);

        let m = ":irc.example.com CAP jwodder ACK message-tags";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        let outgoing = cmd
            .get_client_messages()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["CAP REQ :sasl"]);
        assert!(!cmd.is_done());

        let m = ":irc.example.com CAP jwodder ACK :sasl";
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

        let m = ":irc.example.com 900 jwodder jwodder!jwuser@127.0.0.1 jwodder :You are now logged in as jwodder";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        assert!(cmd.get_client_messages().is_empty());
        assert!(!cmd.is_done());

        let m = ":irc.example.com 903 jwodder :SASL authentication successful";
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
            ":irc.example.com 001 jwodder :Welcome to the Example Internet Relay Chat Network, jwodder",
            ":irc.example.com 002 jwodder :Your host is irc.example.com, running version solanum-1.0-dev",
            ":irc.example.com 003 jwodder :This server was created Thu Jul 18 2024 at 16:57:02 UTC",
            ":irc.example.com 004 jwodder irc.example.com solanum-1.0-dev DGIMQRSZaghilopsuwz CFILMPQRSTbcefgijklmnopqrstuvz bkloveqjfI",
            ":irc.example.com 005 jwodder ACCOUNTEXTBAN=a WHOX KNOCK MONITOR=100 ETRACE FNC SAFELIST ELIST=CMNTU CALLERID=g CHANTYPES=# EXCEPTS INVEX :are supported by this server",
            ":irc.example.com 005 jwodder CHANMODES=eIbq,k,flj,CFLMPQRSTcgimnprstuz CHANLIMIT=#:250 PREFIX=(ov)@+ MAXLIST=bqeI:100 MODES=4 NETWORK=Libera.Chat STATUSMSG=@+ CASEMAPPING=rfc1459 NICKLEN=16 MAXNICKLEN=16 CHANNELLEN=50 TOPICLEN=390 :are supported by this server",
            ":irc.example.com 005 jwodder DEAF=D TARGMAX=NAMES:1,LIST:1,KICK:1,WHOIS:1,PRIVMSG:4,NOTICE:4,ACCEPT:,MONITOR: EXTBAN=$,agjrxz :are supported by this server",
            ":irc.example.com 251 jwodder :There are 62 users and 31502 invisible on 28 servers",
            ":irc.example.com 252 jwodder 40 :IRC Operators online",
            ":irc.example.com 253 jwodder 66 :unknown connection(s)",
            ":irc.example.com 254 jwodder 22798 :channels formed",
            ":irc.example.com 255 jwodder :I have 2700 clients and 1 servers",
            ":irc.example.com 265 jwodder 2700 3071 :Current local users 2700, max 3071",
            ":irc.example.com 266 jwodder 31564 34153 :Current global users 31564, max 34153",
            ":irc.example.com 250 jwodder :Highest connection count: 3072 (3071 clients) (781421 connections received)",
            ":irc.example.com 422 jwodder :No message today",
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
                    (
                        "sasl".parse::<Capability>().unwrap(),
                        Some(
                            "ECDSA-NIST256P-CHALLENGE,EXTERNAL,PLAIN,SCRAM-SHA-512"
                                .parse::<CapabilityValue>()
                                .unwrap()
                        )
                    ),
                    ("message-tags".parse::<Capability>().unwrap(), None),
                ]),
                capabilities_enabled: HashSet::from(["message-tags".parse::<Capability>().unwrap(), "sasl".parse::<Capability>().unwrap()]),
                my_nick: "jwodder".parse::<Nickname>().unwrap(),
                welcome_msg: "Welcome to the Example Internet Relay Chat Network, jwodder".into(),
                yourhost_msg: "Your host is irc.example.com, running version solanum-1.0-dev".into(),
                created_msg: "This server was created Thu Jul 18 2024 at 16:57:02 UTC".into(),
                server_info: ServerInfo {
                    name: "irc.example.com".into(),
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
                    luserclient_msg: Some("There are 62 users and 31502 invisible on 28 servers".to_owned()),
                    luserme_msg: Some("I have 2700 clients and 1 servers".to_owned()),
                    statsconn_msg: Some("Highest connection count: 3072 (3071 clients) (781421 connections received)".to_owned()),
                },
                motd: None,
                mode: Some("+Ziw".parse::<ModeString>().unwrap()),
            }
        );
    }

    #[test]
    fn required_cap_not_supported() {
        let params = LoginParams {
            password: "hunter2".parse::<TrailingParam>().unwrap(),
            nickname: "jwodder".parse::<Nickname>().unwrap(),
            username: "jwuser".parse::<Username>().unwrap(),
            realname: "Just this guy, you know?".parse::<TrailingParam>().unwrap(),
            sasl: true,
            sasl_mechanisms: Vec1::from_one(SaslMechanism::Plain),
            capabilities: BTreeMap::from([
                (
                    "message-tags".parse::<Capability>().unwrap(),
                    CapDesire::Required,
                ),
                (
                    "server-time".parse::<Capability>().unwrap(),
                    CapDesire::Optional,
                ),
            ]),
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
        let m = ":irc.example.com CAP * LS :account-notify away-notify sasl=ECDSA-NIST256P-CHALLENGE,EXTERNAL,PLAIN,SCRAM-SHA-512";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        assert!(cmd.get_client_messages().is_empty());
        assert!(cmd.is_done());
        let output = cmd.get_output().unwrap_err();
        assert_matches::assert_matches!(output, LoginError::RequiredCapNotSupported { capability} => {
            assert_eq!(capability, "message-tags");
        });
    }
}
