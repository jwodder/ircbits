mod admin;
mod authenticate;
mod away;
mod cap;
mod connect;
mod error;
mod help;
mod info;
mod invite;
mod join;
mod kick;
mod kill;
mod links;
mod list;
mod lusers;
mod mode;
mod motd;
mod names;
mod nick;
mod notice;
mod oper;
mod part;
mod pass;
mod ping;
mod pong;
mod privmsg;
mod quit;
mod rehash;
mod restart;
mod squit;
mod stats;
mod time;
mod topic;
mod user;
mod userhost;
mod version;
mod wallops;
mod who;
mod whois;
mod whowas;
pub use self::admin::*;
pub use self::authenticate::*;
pub use self::away::*;
pub use self::cap::*;
pub use self::connect::*;
pub use self::error::*;
pub use self::help::*;
pub use self::info::*;
pub use self::invite::*;
pub use self::join::*;
pub use self::kick::*;
pub use self::kill::*;
pub use self::links::*;
pub use self::list::*;
pub use self::lusers::*;
pub use self::mode::*;
pub use self::motd::*;
pub use self::names::*;
pub use self::nick::*;
pub use self::notice::*;
pub use self::oper::*;
pub use self::part::*;
pub use self::pass::*;
pub use self::ping::*;
pub use self::pong::*;
pub use self::privmsg::*;
pub use self::quit::*;
pub use self::rehash::*;
pub use self::restart::*;
pub use self::squit::*;
pub use self::stats::*;
pub use self::time::*;
pub use self::topic::*;
pub use self::user::*;
pub use self::userhost::*;
pub use self::version::*;
pub use self::wallops::*;
pub use self::who::*;
pub use self::whois::*;
pub use self::whowas::*;
use crate::types::{
    ParseChannelError, ParseEListCondError, ParseKeyError, ParseModeStringError,
    ParseModeTargetError, ParseMsgTargetError, ParseNicknameError, ParseReplyTargetError,
    ParseUsernameError,
};
use crate::{
    Message, ParameterList, ParameterListSizeError, ParseMedialParamError, Payload, RawMessage,
    TryFromStringError, Verb,
};
use enum_dispatch::enum_dispatch;
use thiserror::Error;

#[enum_dispatch]
pub trait ClientMessageParts {
    fn into_parts(self) -> (Verb, ParameterList);

    // Does not include the terminating CR LF
    fn to_irc_line(&self) -> String;
}

#[enum_dispatch(ClientMessageParts)] // This also gives us From and TryInto
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ClientMessage {
    Admin,
    Authenticate,
    Away,
    Cap,
    Connect,
    Error,
    Help,
    Info,
    Invite,
    Join,
    Kick,
    Kill,
    Links,
    List,
    Lusers,
    Mode,
    Motd,
    Names,
    Nick,
    Notice,
    Oper,
    Part,
    Pass,
    Ping,
    Pong,
    PrivMsg,
    Quit,
    Rehash,
    Restart,
    Squit,
    Stats,
    Time,
    Topic,
    User,
    UserHost,
    Version,
    Wallops,
    Who,
    WhoIs,
    WhoWas,
}

impl ClientMessage {
    pub fn from_parts(
        verb: Verb,
        params: ParameterList,
    ) -> Result<ClientMessage, ClientMessageError> {
        match verb {
            Verb::Admin => Admin::try_from(params).map(ClientMessage::Admin),
            Verb::Authenticate => Authenticate::try_from(params).map(ClientMessage::Authenticate),
            Verb::Away => Away::try_from(params).map(ClientMessage::Away),
            Verb::Cap => Cap::try_from(params).map(ClientMessage::Cap),
            Verb::Connect => Connect::try_from(params).map(ClientMessage::Connect),
            Verb::Error => Error::try_from(params).map(ClientMessage::Error),
            Verb::Help => Help::try_from(params).map(ClientMessage::Help),
            Verb::Info => Info::try_from(params).map(ClientMessage::Info),
            Verb::Invite => Invite::try_from(params).map(ClientMessage::Invite),
            Verb::Join => Join::try_from(params).map(ClientMessage::Join),
            Verb::Kick => Kick::try_from(params).map(ClientMessage::Kick),
            Verb::Kill => Kill::try_from(params).map(ClientMessage::Kill),
            Verb::Links => Links::try_from(params).map(ClientMessage::Links),
            Verb::List => List::try_from(params).map(ClientMessage::List),
            Verb::Lusers => Lusers::try_from(params).map(ClientMessage::Lusers),
            Verb::Mode => Mode::try_from(params).map(ClientMessage::Mode),
            Verb::Motd => Motd::try_from(params).map(ClientMessage::Motd),
            Verb::Names => Names::try_from(params).map(ClientMessage::Names),
            Verb::Nick => Nick::try_from(params).map(ClientMessage::Nick),
            Verb::Notice => Notice::try_from(params).map(ClientMessage::Notice),
            Verb::Oper => Oper::try_from(params).map(ClientMessage::Oper),
            Verb::Part => Part::try_from(params).map(ClientMessage::Part),
            Verb::Pass => Pass::try_from(params).map(ClientMessage::Pass),
            Verb::Ping => Ping::try_from(params).map(ClientMessage::Ping),
            Verb::Pong => Pong::try_from(params).map(ClientMessage::Pong),
            Verb::PrivMsg => PrivMsg::try_from(params).map(ClientMessage::PrivMsg),
            Verb::Quit => Quit::try_from(params).map(ClientMessage::Quit),
            Verb::Rehash => Rehash::try_from(params).map(ClientMessage::Rehash),
            Verb::Restart => Restart::try_from(params).map(ClientMessage::Restart),
            Verb::Squit => Squit::try_from(params).map(ClientMessage::Squit),
            Verb::Stats => Stats::try_from(params).map(ClientMessage::Stats),
            Verb::Time => Time::try_from(params).map(ClientMessage::Time),
            Verb::Topic => Topic::try_from(params).map(ClientMessage::Topic),
            Verb::User => User::try_from(params).map(ClientMessage::User),
            Verb::UserHost => UserHost::try_from(params).map(ClientMessage::UserHost),
            Verb::Version => Version::try_from(params).map(ClientMessage::Version),
            Verb::Wallops => Wallops::try_from(params).map(ClientMessage::Wallops),
            Verb::Who => Who::try_from(params).map(ClientMessage::Who),
            Verb::WhoIs => WhoIs::try_from(params).map(ClientMessage::WhoIs),
            Verb::WhoWas => WhoWas::try_from(params).map(ClientMessage::WhoWas),
            Verb::Unknown(v) => Err(ClientMessageError::Unknown(v)),
        }
    }
}

impl From<ClientMessage> for Message {
    fn from(value: ClientMessage) -> Message {
        Message {
            source: None,
            payload: Payload::ClientMessage(value),
        }
    }
}

impl From<ClientMessage> for RawMessage {
    fn from(value: ClientMessage) -> RawMessage {
        RawMessage::from(Message::from(value))
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ClientMessageError {
    #[error("unknown/unrecognized client message verb {0:?}")]
    Unknown(String),

    #[error("unknown/unrecognized CAP subcommand: {0:?}")]
    UnknownCap(String),

    #[error(transparent)]
    ParamQty(#[from] ParameterListSizeError),

    #[error("failed to parse capability string")]
    Capability(#[from] TryFromStringError<ParseCapabilityError>),

    #[error("failed to parse capability value string")]
    CapabilityValue(#[from] TryFromStringError<ParseCapabilityValueError>),

    #[error("failed to parse channel string")]
    Channel(#[from] TryFromStringError<ParseChannelError>),

    #[error("failed to parse elistcond string")]
    EListCond(#[from] TryFromStringError<ParseEListCondError>),

    #[error("failed to parse key string")]
    Key(#[from] TryFromStringError<ParseKeyError>),

    #[error("failed to parse parameter as medial")]
    MedialParam(#[from] TryFromStringError<ParseMedialParamError>),

    #[error("failed to parse mode string")]
    ModeString(#[from] TryFromStringError<ParseModeStringError>),

    #[error("failed to parse mode target string")]
    ModeTarget(#[from] TryFromStringError<ParseModeTargetError>),

    #[error("failed to parse message target string")]
    MsgTarget(#[from] TryFromStringError<ParseMsgTargetError>),

    #[error("failed to parse nickname string")]
    Nickname(#[from] TryFromStringError<ParseNicknameError>),

    #[error("failed to parse reply target string")]
    ReplyTarget(#[from] TryFromStringError<ParseReplyTargetError>),

    #[error("failed to parse username string")]
    Username(#[from] TryFromStringError<ParseUsernameError>),

    #[error("failed to parse integer string {string:?}: {inner}")]
    Int {
        string: String,
        inner: std::num::ParseIntError,
    },

    #[error("parameter has invalid value: expected {expected:?}, got {got:?}")]
    ParamValue { got: String, expected: &'static str },
}
