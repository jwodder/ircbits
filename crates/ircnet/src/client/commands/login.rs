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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
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
                        unexpected => LoginError::UnexpectedError {
                            code: unexpected.code(),
                            reply: msg.to_string(),
                        },
                    };
                    self.state = State::Done(Some(Err(e)));
                    true
                } else {
                    self.state.in_place(|state| state.handle_reply(rpl))
                }
            }
            Payload::ClientMessage(climsg) => match climsg {
                ClientMessage::Error(err) => {
                    self.state = State::Done(Some(Err(LoginError::ErrorMessage {
                        reason: err.reason().to_string(),
                    })));
                    true
                }
                ClientMessage::Mode(mode) => self.state.in_place(|state| state.handle_mode(mode)),
                ClientMessage::Ping(_) | ClientMessage::PrivMsg(_) | ClientMessage::Notice(_) => {
                    false
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
    fn in_place<F>(&mut self, f: F) -> bool
    where
        F: FnOnce(Self) -> Result<(State, bool), LoginError>,
    {
        let state = std::mem::replace(self, State::Void);
        match f(state) {
            Ok((st, b)) => {
                *self = st;
                b
            }
            Err(e) => {
                *self = State::Done(Some(Err(e)));
                true
            }
        }
    }

    fn handle_reply(self, rpl: &Reply) -> Result<(State, bool), LoginError> {
        match (self, rpl) {
            (State::Start, Reply::Welcome(r)) => {
                if let ReplyTarget::Nick(nick) = r.client() {
                    let my_nick = nick.clone();
                    Ok((State::Got001 { my_nick }, true))
                } else {
                    Err(LoginError::StarWelcome)
                }
            }
            (State::Got001 { my_nick }, Reply::YourHost(_)) => {
                Ok((State::Got002 { my_nick }, true))
            }
            (State::Got002 { my_nick }, Reply::Created(_)) => Ok((State::Got003 { my_nick }, true)),
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
                Ok((State::Got004(output), true))
            }
            (State::Got004(mut output) | State::Got005(mut output), Reply::ISupport(r)) => {
                output.isupport.extend(r.tokens().iter().cloned());
                Ok((State::Got005(output), true))
            }
            (State::Got005(output) | State::Lusers(output), Reply::StatsConn(_)) => {
                Ok((State::Lusers(output), true))
            }
            (State::Got005(output) | State::Lusers(output), Reply::LuserClient(_)) => {
                Ok((State::Lusers(output), true))
            }
            (State::Got005(mut output) | State::Lusers(mut output), Reply::LuserOp(r)) => {
                output.luser_stats.operators = Some(r.ops());
                Ok((State::Lusers(output), true))
            }
            (State::Got005(mut output) | State::Lusers(mut output), Reply::LuserUnknown(r)) => {
                output.luser_stats.unknown_connections = Some(r.connections());
                Ok((State::Lusers(output), true))
            }
            (State::Got005(mut output) | State::Lusers(mut output), Reply::LuserChannels(r)) => {
                output.luser_stats.channels = Some(r.channels());
                Ok((State::Lusers(output), true))
            }
            (State::Got005(output) | State::Lusers(output), Reply::LuserMe(_)) => {
                Ok((State::Lusers(output), true))
            }
            (State::Got005(mut output) | State::Lusers(mut output), Reply::LocalUsers(r)) => {
                output.luser_stats.local_clients = r.current_users();
                output.luser_stats.max_local_clients = r.max_users();
                Ok((State::Lusers(output), true))
            }
            (State::Got005(mut output) | State::Lusers(mut output), Reply::GlobalUsers(r)) => {
                output.luser_stats.global_clients = r.current_users();
                output.luser_stats.max_global_clients = r.max_users();
                Ok((State::Lusers(output), true))
            }
            (State::Got005(mut output) | State::Lusers(mut output), Reply::MotdStart(r)) => {
                output.motd = Some(r.message().to_owned());
                Ok((State::Motd(output), true))
            }
            (State::Got005(mut output) | State::Lusers(mut output), Reply::NoMotd(r)) => {
                output.motd = Some(r.message().to_owned());
                Ok((
                    State::AwaitingMode {
                        output,
                        timeout: Some(MODE_TIMEOUT),
                    },
                    true,
                ))
            }
            (st @ State::Got005(_), _) => Ok((st, false)), // Accept "other numerics and messages" after RPL_ISUPPORT
            (State::Motd(mut output), Reply::Motd(r)) => {
                if let Some(s) = output.motd.as_mut() {
                    s.push('\n');
                    s.push_str(r.message());
                }
                Ok((State::Motd(output), true))
            }
            (State::Motd(mut output), Reply::EndOfMotd(r)) => {
                if let Some(s) = output.motd.as_mut() {
                    s.push('\n');
                    s.push_str(r.message());
                }
                Ok((
                    State::AwaitingMode {
                        output,
                        timeout: Some(MODE_TIMEOUT),
                    },
                    true,
                ))
            }
            (State::AwaitingMode { mut output, .. }, Reply::UModeIs(r)) => {
                let ms = r.user_modes();
                let ms = if ms.starts_with(['+', '-']) {
                    ms.to_owned()
                } else {
                    format!("+{ms}")
                };
                let Ok(modestring) = ms.parse::<ModeString>() else {
                    return Err(LoginError::InvalidMode {
                        msg: r.to_irc_line(),
                    });
                };
                output.mode = Some(modestring);
                Ok((State::Done(Some(Ok(output))), true))
            }
            (st @ State::Done(_), _) => Ok((st, false)),
            (State::Void, _) => panic!("handle_reply() called on Void login state"),
            (st, other) => {
                let expecting = st.expecting();
                let msg = other.to_irc_line();
                Err(LoginError::Unexpected { expecting, msg })
            }
        }
    }

    fn handle_mode(self, mode: &Mode) -> Result<(State, bool), LoginError> {
        match self {
            State::AwaitingMode { mut output, .. } => {
                output.mode = mode.modestring().cloned();
                Ok((State::Done(Some(Ok(output))), true))
            }
            State::Void => panic!("handle_mode() called on Void login state"),
            st => {
                let expecting = st.expecting();
                let msg = mode.to_irc_line();
                Err(LoginError::Unexpected { expecting, msg })
            }
        }
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
                *self = State::Done(Some(Err(LoginError::Unexpected { expecting, msg })));
                true
            }
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
            State::Void => panic!("expecting() called on Void login state"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoginOutput {
    // SASL: CAP LS
    pub my_nick: Nickname,
    pub server_info: ServerInfo,
    pub isupport: Vec<ISupportParam>,
    pub luser_stats: LuserStats,
    pub motd: Option<String>, // None if the server reports no MOTD was set
    pub mode: Option<ModeString>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerInfo {
    pub server_name: String,
    pub version: String,
    pub user_modes: String,
    pub channel_modes: String,
    pub param_channel_modes: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LuserStats {
    pub operators: Option<u64>,
    pub unknown_connections: Option<u64>,
    pub channels: Option<u64>,
    pub local_clients: Option<u64>,
    pub max_local_clients: Option<u64>,
    pub global_clients: Option<u64>,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_login() {
        let params = LoginParams {
            password: "hunter2".parse::<FinalParam>().unwrap(),
            nickname: "jwodder".parse::<Nickname>().unwrap(),
            username: "jwuser".parse::<Username>().unwrap(),
            realname: "Just this guy, you know?".parse::<FinalParam>().unwrap(),
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
                my_nick: "jwodder".parse::<Nickname>().unwrap(),
                server_info: ServerInfo {
                    server_name: "molybdenum.libera.chat".into(),
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
