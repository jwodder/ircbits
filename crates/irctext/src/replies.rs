use crate::types::{
    Channel, ChannelStatus, ISupportParam, ModeString, ModeTarget, MsgTarget, Nickname,
    ParseChannelError, ParseChannelStatusError, ParseISupportParamError, ParseModeStringError,
    ParseModeTargetError, ParseMsgTargetError, ParseNicknameError, ParseReplyTargetError,
    ParseUserHostReplyError, ParseUsernameError, ParseWhoFlagsError, ReplyTarget, UserHostReply,
    Username, WhoFlags,
};
use crate::util::{pop_channel_membership, split_spaces, split_word};
use crate::{
    ClientSource, Message, ParameterList, ParseClientSourceError, ParseVerbError, Payload,
    RawMessage, TryFromStringError, Verb,
};
use enum_dispatch::enum_dispatch;
use std::net::IpAddr;
use thiserror::Error;
use url::Host;

#[enum_dispatch]
pub trait ReplyParts {
    fn code(&self) -> u16;
    fn parameters(&self) -> &ParameterList;
    fn is_error(&self) -> bool;
    fn into_parts(self) -> (u16, ParameterList);
}

#[enum_dispatch(ReplyParts)] // This also gives us From and TryInto
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Reply {
    Welcome,
    YourHost,
    Created,
    MyInfo,
    ISupport,
    RemoteISupport,
    Bounce,
    StatsCommands,
    EndOfStats,
    UModeIs,
    StatsUptime,
    LuserClient,
    LuserOp,
    LuserUnknown,
    LuserChannels,
    LuserMe,
    AdminMe,
    AdminLoc1,
    AdminLoc2,
    AdminEmail,
    TryAgain,
    LocalUsers,
    GlobalUsers,
    WhoIsCertFP,
    None,
    Away,
    UserHostRpl,
    UnAway,
    NowAway,
    WhoIsRegNick,
    WhoIsUser,
    WhoIsServer,
    WhoIsOperator,
    WhoWasUser,
    EndOfWho,
    WhoIsIdle,
    EndOfWhoIs,
    WhoIsChannels,
    WhoIsSpecial,
    ListStart,
    List,
    ListEnd,
    ChannelModeIs,
    CreationTime,
    WhoIsAccount,
    NoTopic,
    Topic,
    TopicWhoTime,
    InviteList,
    EndOfInviteList,
    WhoIsActually,
    Inviting,
    InvExList,
    EndOfInvExList,
    ExceptList,
    EndOfExceptList,
    Version,
    WhoReply,
    NamReply,
    Links,
    EndOfLinks,
    EndOfNames,
    BanList,
    EndOfBanList,
    EndOfWhoWas,
    Info,
    Motd,
    EndOfInfo,
    MotdStart,
    EndOfMotd,
    WhoIsHost,
    WhoIsModes,
    YoureOper,
    Rehashing,
    Time,
    UnknownError,
    NoSuchNick,
    NoSuchServer,
    NoSuchChannel,
    CannotSendToChan,
    TooManyChannels,
    WasNoSuchNick,
    NoOrigin,
    NoRecipient,
    NoTextToSend,
    InputTooLong,
    UnknownCommand,
    NoMotd,
    NoNicknameGiven,
    ErroneousNickname,
    NicknameInUse,
    NickCollision,
    UserNotInChannel,
    NotOnChannel,
    UserOnChannel,
    NotRegistered,
    NeedMoreParams,
    AlreadyRegistered,
    PasswdMismatch,
    YoureBannedCreep,
    ChannelIsFull,
    UnknownMode,
    InviteOnlyChan,
    BannedFromChan,
    BadChannelKey,
    BadChanMask,
    NoPrivileges,
    ChanOPrivsNeeded,
    CantKillServer,
    NoOperHost,
    UmodeUnknownFlag,
    UsersDontMatch,
    HelpNotFound,
    InvalidKey,
    StartTLS,
    WhoIsSecure,
    StartTLSError,
    InvalidModeParam,
    HelpStart,
    HelpTxt,
    EndOfHelp,
    NoPrivs,
    LoggedIn,
    LoggedOut,
    NickLocked,
    SaslSuccess,
    SaslFail,
    SaslTooLong,
    SaslAborted,
    SaslAlready,
    SaslMechs,
}

impl Reply {
    pub fn from_parts(code: u16, params: ParameterList) -> Result<Reply, ReplyError> {
        match code {
            1 => Welcome::try_from(params).map(Into::into),
            2 => YourHost::try_from(params).map(Into::into),
            3 => Created::try_from(params).map(Into::into),
            4 => MyInfo::try_from(params).map(Into::into),
            5 => ISupport::try_from(params).map(Into::into),
            105 => RemoteISupport::try_from(params).map(Into::into),
            10 => Bounce::try_from(params).map(Into::into),
            212 => StatsCommands::try_from(params).map(Into::into),
            219 => EndOfStats::try_from(params).map(Into::into),
            221 => UModeIs::try_from(params).map(Into::into),
            242 => StatsUptime::try_from(params).map(Into::into),
            251 => LuserClient::try_from(params).map(Into::into),
            252 => LuserOp::try_from(params).map(Into::into),
            253 => LuserUnknown::try_from(params).map(Into::into),
            254 => LuserChannels::try_from(params).map(Into::into),
            255 => LuserMe::try_from(params).map(Into::into),
            256 => AdminMe::try_from(params).map(Into::into),
            257 => AdminLoc1::try_from(params).map(Into::into),
            258 => AdminLoc2::try_from(params).map(Into::into),
            259 => AdminEmail::try_from(params).map(Into::into),
            263 => TryAgain::try_from(params).map(Into::into),
            265 => LocalUsers::try_from(params).map(Into::into),
            266 => GlobalUsers::try_from(params).map(Into::into),
            276 => WhoIsCertFP::try_from(params).map(Into::into),
            300 => None::try_from(params).map(Into::into),
            301 => Away::try_from(params).map(Into::into),
            302 => UserHostRpl::try_from(params).map(Into::into),
            305 => UnAway::try_from(params).map(Into::into),
            306 => NowAway::try_from(params).map(Into::into),
            307 => WhoIsRegNick::try_from(params).map(Into::into),
            311 => WhoIsUser::try_from(params).map(Into::into),
            312 => WhoIsServer::try_from(params).map(Into::into),
            313 => WhoIsOperator::try_from(params).map(Into::into),
            314 => WhoWasUser::try_from(params).map(Into::into),
            315 => EndOfWho::try_from(params).map(Into::into),
            317 => WhoIsIdle::try_from(params).map(Into::into),
            318 => EndOfWhoIs::try_from(params).map(Into::into),
            319 => WhoIsChannels::try_from(params).map(Into::into),
            320 => WhoIsSpecial::try_from(params).map(Into::into),
            321 => ListStart::try_from(params).map(Into::into),
            322 => List::try_from(params).map(Into::into),
            323 => ListEnd::try_from(params).map(Into::into),
            324 => ChannelModeIs::try_from(params).map(Into::into),
            329 => CreationTime::try_from(params).map(Into::into),
            330 => WhoIsAccount::try_from(params).map(Into::into),
            331 => NoTopic::try_from(params).map(Into::into),
            332 => Topic::try_from(params).map(Into::into),
            333 => TopicWhoTime::try_from(params).map(Into::into),
            336 => InviteList::try_from(params).map(Into::into),
            337 => EndOfInviteList::try_from(params).map(Into::into),
            338 => WhoIsActually::try_from(params).map(Into::into),
            341 => Inviting::try_from(params).map(Into::into),
            346 => InvExList::try_from(params).map(Into::into),
            347 => EndOfInvExList::try_from(params).map(Into::into),
            348 => ExceptList::try_from(params).map(Into::into),
            349 => EndOfExceptList::try_from(params).map(Into::into),
            351 => Version::try_from(params).map(Into::into),
            352 => WhoReply::try_from(params).map(Into::into),
            353 => NamReply::try_from(params).map(Into::into),
            364 => Links::try_from(params).map(Into::into),
            365 => EndOfLinks::try_from(params).map(Into::into),
            366 => EndOfNames::try_from(params).map(Into::into),
            367 => BanList::try_from(params).map(Into::into),
            368 => EndOfBanList::try_from(params).map(Into::into),
            369 => EndOfWhoWas::try_from(params).map(Into::into),
            371 => Info::try_from(params).map(Into::into),
            372 => Motd::try_from(params).map(Into::into),
            374 => EndOfInfo::try_from(params).map(Into::into),
            375 => MotdStart::try_from(params).map(Into::into),
            376 => EndOfMotd::try_from(params).map(Into::into),
            378 => WhoIsHost::try_from(params).map(Into::into),
            379 => WhoIsModes::try_from(params).map(Into::into),
            381 => YoureOper::try_from(params).map(Into::into),
            382 => Rehashing::try_from(params).map(Into::into),
            391 => Time::try_from(params).map(Into::into),
            400 => UnknownError::try_from(params).map(Into::into),
            401 => NoSuchNick::try_from(params).map(Into::into),
            402 => NoSuchServer::try_from(params).map(Into::into),
            403 => NoSuchChannel::try_from(params).map(Into::into),
            404 => CannotSendToChan::try_from(params).map(Into::into),
            405 => TooManyChannels::try_from(params).map(Into::into),
            406 => WasNoSuchNick::try_from(params).map(Into::into),
            409 => NoOrigin::try_from(params).map(Into::into),
            411 => NoRecipient::try_from(params).map(Into::into),
            412 => NoTextToSend::try_from(params).map(Into::into),
            417 => InputTooLong::try_from(params).map(Into::into),
            421 => UnknownCommand::try_from(params).map(Into::into),
            422 => NoMotd::try_from(params).map(Into::into),
            431 => NoNicknameGiven::try_from(params).map(Into::into),
            432 => ErroneousNickname::try_from(params).map(Into::into),
            433 => NicknameInUse::try_from(params).map(Into::into),
            436 => NickCollision::try_from(params).map(Into::into),
            441 => UserNotInChannel::try_from(params).map(Into::into),
            442 => NotOnChannel::try_from(params).map(Into::into),
            443 => UserOnChannel::try_from(params).map(Into::into),
            451 => NotRegistered::try_from(params).map(Into::into),
            461 => NeedMoreParams::try_from(params).map(Into::into),
            462 => AlreadyRegistered::try_from(params).map(Into::into),
            464 => PasswdMismatch::try_from(params).map(Into::into),
            465 => YoureBannedCreep::try_from(params).map(Into::into),
            471 => ChannelIsFull::try_from(params).map(Into::into),
            472 => UnknownMode::try_from(params).map(Into::into),
            473 => InviteOnlyChan::try_from(params).map(Into::into),
            474 => BannedFromChan::try_from(params).map(Into::into),
            475 => BadChannelKey::try_from(params).map(Into::into),
            476 => BadChanMask::try_from(params).map(Into::into),
            481 => NoPrivileges::try_from(params).map(Into::into),
            482 => ChanOPrivsNeeded::try_from(params).map(Into::into),
            483 => CantKillServer::try_from(params).map(Into::into),
            491 => NoOperHost::try_from(params).map(Into::into),
            501 => UmodeUnknownFlag::try_from(params).map(Into::into),
            502 => UsersDontMatch::try_from(params).map(Into::into),
            524 => HelpNotFound::try_from(params).map(Into::into),
            525 => InvalidKey::try_from(params).map(Into::into),
            670 => StartTLS::try_from(params).map(Into::into),
            671 => WhoIsSecure::try_from(params).map(Into::into),
            691 => StartTLSError::try_from(params).map(Into::into),
            696 => InvalidModeParam::try_from(params).map(Into::into),
            704 => HelpStart::try_from(params).map(Into::into),
            705 => HelpTxt::try_from(params).map(Into::into),
            706 => EndOfHelp::try_from(params).map(Into::into),
            723 => NoPrivs::try_from(params).map(Into::into),
            900 => LoggedIn::try_from(params).map(Into::into),
            901 => LoggedOut::try_from(params).map(Into::into),
            902 => NickLocked::try_from(params).map(Into::into),
            903 => SaslSuccess::try_from(params).map(Into::into),
            904 => SaslFail::try_from(params).map(Into::into),
            905 => SaslTooLong::try_from(params).map(Into::into),
            906 => SaslAborted::try_from(params).map(Into::into),
            907 => SaslAlready::try_from(params).map(Into::into),
            908 => SaslMechs::try_from(params).map(Into::into),
            _ => Err(ReplyError::Unknown(code)),
        }
    }
}

impl From<Reply> for Message {
    fn from(value: Reply) -> Message {
        Message {
            source: None,
            payload: Payload::Reply(value),
        }
    }
}

impl From<Reply> for RawMessage {
    fn from(value: Reply) -> RawMessage {
        RawMessage::from(Message::from(value))
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ReplyError {
    #[error("unknown/unrecognized reply code {0:03}")]
    Unknown(u16),

    #[error("invalid number of parameters: at least {min_required} required, {received} received")]
    ParamQty {
        min_required: usize,
        received: usize,
    },

    #[error("failed to parse host string {string:?}: {inner}")]
    Host {
        string: String,
        inner: url::ParseError,
    },

    #[error("failed to parse integer string {string:?}: {inner}")]
    Int {
        string: String,
        inner: std::num::ParseIntError,
    },

    #[error("failed to parse IP address string {string:?}: {inner}")]
    IpAddr {
        string: String,
        inner: std::net::AddrParseError,
    },

    #[error("failed to parse channel string")]
    Channel(#[from] TryFromStringError<ParseChannelError>),

    #[error("failed to parse channel status string")]
    ChannelStatus(#[from] TryFromStringError<ParseChannelStatusError>),

    #[error("failed to parse source string")]
    ClientSource(#[from] TryFromStringError<ParseClientSourceError>),

    #[error("failed to parse RPL_ISUPPORT param")]
    ISupportParam(#[from] TryFromStringError<ParseISupportParamError>),

    #[error("failed to parse mode string")]
    ModeString(#[from] TryFromStringError<ParseModeStringError>),

    #[error("failed to parse mode target string")]
    ModeTarget(#[from] TryFromStringError<ParseModeTargetError>),

    #[error("failed to parse target string")]
    MsgTarget(#[from] TryFromStringError<ParseMsgTargetError>),

    #[error("failed to parse nickname string")]
    Nickname(#[from] TryFromStringError<ParseNicknameError>),

    #[error("failed to parse reply target string")]
    ReplyTarget(#[from] TryFromStringError<ParseReplyTargetError>),

    #[error("failed to parse USERHOST reply string")]
    UserHostReply(#[from] TryFromStringError<ParseUserHostReplyError>),

    #[error("failed to parse username string")]
    Username(#[from] TryFromStringError<ParseUsernameError>),

    #[error("failed to parse verb string")]
    Verb(#[from] TryFromStringError<ParseVerbError>),

    #[error("failed to parse RPL_WHOREPLY flags")]
    WhoFlags(#[from] TryFromStringError<ParseWhoFlagsError>),

    #[error("invalid user@host string: {0:?}: expected '@'")]
    NoAt(String),
}

pub mod codes {
    pub const RPL_WELCOME: u16 = 1;
    pub const RPL_YOURHOST: u16 = 2;
    pub const RPL_CREATED: u16 = 3;
    pub const RPL_MYINFO: u16 = 4;
    pub const RPL_ISUPPORT: u16 = 5;
    pub const RPL_REMOTEISUPPORT: u16 = 105;
    pub const RPL_BOUNCE: u16 = 10;
    pub const RPL_STATSCOMMANDS: u16 = 212;
    pub const RPL_ENDOFSTATS: u16 = 219;
    pub const RPL_UMODEIS: u16 = 221;
    pub const RPL_STATSUPTIME: u16 = 242;
    pub const RPL_LUSERCLIENT: u16 = 251;
    pub const RPL_LUSEROP: u16 = 252;
    pub const RPL_LUSERUNKNOWN: u16 = 253;
    pub const RPL_LUSERCHANNELS: u16 = 254;
    pub const RPL_LUSERME: u16 = 255;
    pub const RPL_ADMINME: u16 = 256;
    pub const RPL_ADMINLOC1: u16 = 257;
    pub const RPL_ADMINLOC2: u16 = 258;
    pub const RPL_ADMINEMAIL: u16 = 259;
    pub const RPL_TRYAGAIN: u16 = 263;
    pub const RPL_LOCALUSERS: u16 = 265;
    pub const RPL_GLOBALUSERS: u16 = 266;
    pub const RPL_WHOISCERTFP: u16 = 276;
    pub const RPL_NONE: u16 = 300;
    pub const RPL_AWAY: u16 = 301;
    pub const RPL_USERHOST: u16 = 302;
    pub const RPL_UNAWAY: u16 = 305;
    pub const RPL_NOWAWAY: u16 = 306;
    pub const RPL_WHOISREGNICK: u16 = 307;
    pub const RPL_WHOISUSER: u16 = 311;
    pub const RPL_WHOISSERVER: u16 = 312;
    pub const RPL_WHOISOPERATOR: u16 = 313;
    pub const RPL_WHOWASUSER: u16 = 314;
    pub const RPL_ENDOFWHO: u16 = 315;
    pub const RPL_WHOISIDLE: u16 = 317;
    pub const RPL_ENDOFWHOIS: u16 = 318;
    pub const RPL_WHOISCHANNELS: u16 = 319;
    pub const RPL_WHOISSPECIAL: u16 = 320;
    pub const RPL_LISTSTART: u16 = 321;
    pub const RPL_LIST: u16 = 322;
    pub const RPL_LISTEND: u16 = 323;
    pub const RPL_CHANNELMODEIS: u16 = 324;
    pub const RPL_CREATIONTIME: u16 = 329;
    pub const RPL_WHOISACCOUNT: u16 = 330;
    pub const RPL_NOTOPIC: u16 = 331;
    pub const RPL_TOPIC: u16 = 332;
    pub const RPL_TOPICWHOTIME: u16 = 333;
    pub const RPL_INVITELIST: u16 = 336;
    pub const RPL_ENDOFINVITELIST: u16 = 337;
    pub const RPL_WHOISACTUALLY: u16 = 338;
    pub const RPL_INVITING: u16 = 341;
    pub const RPL_INVEXLIST: u16 = 346;
    pub const RPL_ENDOFINVEXLIST: u16 = 347;
    pub const RPL_EXCEPTLIST: u16 = 348;
    pub const RPL_ENDOFEXCEPTLIST: u16 = 349;
    pub const RPL_VERSION: u16 = 351;
    pub const RPL_WHOREPLY: u16 = 352;
    pub const RPL_NAMREPLY: u16 = 353;
    pub const RPL_LINKS: u16 = 364;
    pub const RPL_ENDOFLINKS: u16 = 365;
    pub const RPL_ENDOFNAMES: u16 = 366;
    pub const RPL_BANLIST: u16 = 367;
    pub const RPL_ENDOFBANLIST: u16 = 368;
    pub const RPL_ENDOFWHOWAS: u16 = 369;
    pub const RPL_INFO: u16 = 371;
    pub const RPL_MOTD: u16 = 372;
    pub const RPL_ENDOFINFO: u16 = 374;
    pub const RPL_MOTDSTART: u16 = 375;
    pub const RPL_ENDOFMOTD: u16 = 376;
    pub const RPL_WHOISHOST: u16 = 378;
    pub const RPL_WHOISMODES: u16 = 379;
    pub const RPL_YOUREOPER: u16 = 381;
    pub const RPL_REHASHING: u16 = 382;
    pub const RPL_TIME: u16 = 391;
    pub const ERR_UNKNOWNERROR: u16 = 400;
    pub const ERR_NOSUCHNICK: u16 = 401;
    pub const ERR_NOSUCHSERVER: u16 = 402;
    pub const ERR_NOSUCHCHANNEL: u16 = 403;
    pub const ERR_CANNOTSENDTOCHAN: u16 = 404;
    pub const ERR_TOOMANYCHANNELS: u16 = 405;
    pub const ERR_WASNOSUCHNICK: u16 = 406;
    pub const ERR_NOORIGIN: u16 = 409;
    pub const ERR_NORECIPIENT: u16 = 411;
    pub const ERR_NOTEXTTOSEND: u16 = 412;
    pub const ERR_INPUTTOOLONG: u16 = 417;
    pub const ERR_UNKNOWNCOMMAND: u16 = 421;
    pub const ERR_NOMOTD: u16 = 422;
    pub const ERR_NONICKNAMEGIVEN: u16 = 431;
    pub const ERR_ERRONEUSNICKNAME: u16 = 432;
    pub const ERR_NICKNAMEINUSE: u16 = 433;
    pub const ERR_NICKCOLLISION: u16 = 436;
    pub const ERR_USERNOTINCHANNEL: u16 = 441;
    pub const ERR_NOTONCHANNEL: u16 = 442;
    pub const ERR_USERONCHANNEL: u16 = 443;
    pub const ERR_NOTREGISTERED: u16 = 451;
    pub const ERR_NEEDMOREPARAMS: u16 = 461;
    pub const ERR_ALREADYREGISTERED: u16 = 462;
    pub const ERR_PASSWDMISMATCH: u16 = 464;
    pub const ERR_YOUREBANNEDCREEP: u16 = 465;
    pub const ERR_CHANNELISFULL: u16 = 471;
    pub const ERR_UNKNOWNMODE: u16 = 472;
    pub const ERR_INVITEONLYCHAN: u16 = 473;
    pub const ERR_BANNEDFROMCHAN: u16 = 474;
    pub const ERR_BADCHANNELKEY: u16 = 475;
    pub const ERR_BADCHANMASK: u16 = 476;
    pub const ERR_NOPRIVILEGES: u16 = 481;
    pub const ERR_CHANOPRIVSNEEDED: u16 = 482;
    pub const ERR_CANTKILLSERVER: u16 = 483;
    pub const ERR_NOOPERHOST: u16 = 491;
    pub const ERR_UMODEUNKNOWNFLAG: u16 = 501;
    pub const ERR_USERSDONTMATCH: u16 = 502;
    pub const ERR_HELPNOTFOUND: u16 = 524;
    pub const ERR_INVALIDKEY: u16 = 525;
    pub const RPL_STARTTLS: u16 = 670;
    pub const RPL_WHOISSECURE: u16 = 671;
    pub const ERR_STARTTLSERROR: u16 = 691;
    pub const ERR_INVALIDMODEPARAM: u16 = 696;
    pub const RPL_HELPSTART: u16 = 704;
    pub const RPL_HELPTXT: u16 = 705;
    pub const RPL_ENDOFHELP: u16 = 706;
    pub const ERR_NOPRIVS: u16 = 723;
    pub const RPL_LOGGEDIN: u16 = 900;
    pub const RPL_LOGGEDOUT: u16 = 901;
    pub const ERR_NICKLOCKED: u16 = 902;
    pub const RPL_SASLSUCCESS: u16 = 903;
    pub const ERR_SASLFAIL: u16 = 904;
    pub const ERR_SASLTOOLONG: u16 = 905;
    pub const ERR_SASLABORTED: u16 = 906;
    pub const ERR_SASLALREADY: u16 = 907;
    pub const RPL_SASLMECHS: u16 = 908;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Welcome {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl Welcome {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for Welcome {
    fn code(&self) -> u16 {
        codes::RPL_WELCOME
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<Welcome> for Message {
    fn from(value: Welcome) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<Welcome> for RawMessage {
    fn from(value: Welcome) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for Welcome {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<Welcome, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(Welcome { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YourHost {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl YourHost {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for YourHost {
    fn code(&self) -> u16 {
        codes::RPL_YOURHOST
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<YourHost> for Message {
    fn from(value: YourHost) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<YourHost> for RawMessage {
    fn from(value: YourHost) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for YourHost {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<YourHost, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(YourHost { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Created {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl Created {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for Created {
    fn code(&self) -> u16 {
        codes::RPL_CREATED
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<Created> for Message {
    fn from(value: Created) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<Created> for RawMessage {
    fn from(value: Created) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for Created {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<Created, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(Created { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MyInfo {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl MyInfo {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn servername(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn version(&self) -> &str {
        let Some(p) = self.parameters.get(2) else {
            unreachable!("index 2 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn available_user_modes(&self) -> &str {
        let Some(p) = self.parameters.get(3) else {
            unreachable!("index 3 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn available_channel_modes(&self) -> &str {
        let Some(p) = self.parameters.get(4) else {
            unreachable!("index 4 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn channel_modes_with_param(&self) -> Option<&str> {
        self.parameters.get(5).map(|p| p.as_str())
    }
}

impl ReplyParts for MyInfo {
    fn code(&self) -> u16 {
        codes::RPL_MYINFO
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<MyInfo> for Message {
    fn from(value: MyInfo) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<MyInfo> for RawMessage {
    fn from(value: MyInfo) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for MyInfo {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<MyInfo, ReplyError> {
        if parameters.len() < 5 {
            return Err(ReplyError::ParamQty {
                min_required: 5,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 5");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(MyInfo { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ISupport {
    parameters: ParameterList,
    client: ReplyTarget,
    tokens: Vec<ISupportParam>,
}

impl ISupport {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn tokens(&self) -> &[ISupportParam] {
        &self.tokens
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for ISupport {
    fn code(&self) -> u16 {
        codes::RPL_ISUPPORT
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<ISupport> for Message {
    fn from(value: ISupport) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<ISupport> for RawMessage {
    fn from(value: ISupport) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for ISupport {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<ISupport, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let tokens = parameters
            .iter()
            .skip(1)
            .take(parameters.len() - 2)
            .map(|p| ISupportParam::try_from(String::from(p)))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ISupport {
            parameters,
            client,
            tokens,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteISupport {
    parameters: ParameterList,
    client: ReplyTarget,
    tokens: Vec<ISupportParam>,
}

impl RemoteISupport {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn tokens(&self) -> &[ISupportParam] {
        &self.tokens
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for RemoteISupport {
    fn code(&self) -> u16 {
        codes::RPL_REMOTEISUPPORT
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<RemoteISupport> for Message {
    fn from(value: RemoteISupport) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<RemoteISupport> for RawMessage {
    fn from(value: RemoteISupport) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for RemoteISupport {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<RemoteISupport, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let tokens = parameters
            .iter()
            .skip(1)
            .take(parameters.len() - 2)
            .map(|p| ISupportParam::try_from(String::from(p)))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RemoteISupport {
            parameters,
            client,
            tokens,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bounce {
    parameters: ParameterList,
    client: ReplyTarget,
    port: u16,
}

impl Bounce {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn hostname(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for Bounce {
    fn code(&self) -> u16 {
        codes::RPL_BOUNCE
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<Bounce> for Message {
    fn from(value: Bounce) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<Bounce> for RawMessage {
    fn from(value: Bounce) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for Bounce {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<Bounce, ReplyError> {
        if parameters.len() < 4 {
            return Err(ReplyError::ParamQty {
                min_required: 4,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 4");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(2)
            .expect("Parameter 2 should exist when list length is at least 4");
        let port = match p.as_str().parse::<u16>() {
            Ok(n) => n,
            Err(inner) => {
                return Err(ReplyError::Int {
                    string: String::from(p),
                    inner,
                })
            }
        };
        Ok(Bounce {
            parameters,
            client,
            port,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatsCommands {
    parameters: ParameterList,
    client: ReplyTarget,
    count: u64,
    byte_count: Option<u64>,
    remote_count: Option<u64>,
}

impl StatsCommands {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn command(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn byte_count(&self) -> Option<u64> {
        self.byte_count
    }

    pub fn remote_count(&self) -> Option<u64> {
        self.remote_count
    }
}

impl ReplyParts for StatsCommands {
    fn code(&self) -> u16 {
        codes::RPL_STATSCOMMANDS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<StatsCommands> for Message {
    fn from(value: StatsCommands) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<StatsCommands> for RawMessage {
    fn from(value: StatsCommands) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for StatsCommands {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<StatsCommands, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(2)
            .expect("Parameter 2 should exist when list length is at least 3");
        let count = match p.as_str().parse::<u64>() {
            Ok(n) => n,
            Err(inner) => {
                return Err(ReplyError::Int {
                    string: String::from(p),
                    inner,
                })
            }
        };
        let byte_count = parameters
            .get(3)
            .map(|p| match p.as_str().parse::<u64>() {
                Ok(n) => Ok(n),
                Err(inner) => Err(ReplyError::Int {
                    string: String::from(p),
                    inner,
                }),
            })
            .transpose()?;
        let remote_count = parameters
            .get(4)
            .map(|p| match p.as_str().parse::<u64>() {
                Ok(n) => Ok(n),
                Err(inner) => Err(ReplyError::Int {
                    string: String::from(p),
                    inner,
                }),
            })
            .transpose()?;
        Ok(StatsCommands {
            parameters,
            client,
            count,
            byte_count,
            remote_count,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndOfStats {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl EndOfStats {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn stats_letter(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for EndOfStats {
    fn code(&self) -> u16 {
        codes::RPL_ENDOFSTATS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<EndOfStats> for Message {
    fn from(value: EndOfStats) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<EndOfStats> for RawMessage {
    fn from(value: EndOfStats) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for EndOfStats {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<EndOfStats, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(EndOfStats { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UModeIs {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl UModeIs {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn user_modes(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }
}

impl ReplyParts for UModeIs {
    fn code(&self) -> u16 {
        codes::RPL_UMODEIS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<UModeIs> for Message {
    fn from(value: UModeIs) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<UModeIs> for RawMessage {
    fn from(value: UModeIs) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for UModeIs {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<UModeIs, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(UModeIs { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatsUptime {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl StatsUptime {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for StatsUptime {
    fn code(&self) -> u16 {
        codes::RPL_STATSUPTIME
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<StatsUptime> for Message {
    fn from(value: StatsUptime) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<StatsUptime> for RawMessage {
    fn from(value: StatsUptime) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for StatsUptime {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<StatsUptime, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(StatsUptime { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LuserClient {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl LuserClient {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for LuserClient {
    fn code(&self) -> u16 {
        codes::RPL_LUSERCLIENT
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<LuserClient> for Message {
    fn from(value: LuserClient) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<LuserClient> for RawMessage {
    fn from(value: LuserClient) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for LuserClient {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<LuserClient, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(LuserClient { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LuserOp {
    parameters: ParameterList,
    client: ReplyTarget,
    ops: u64,
}

impl LuserOp {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn ops(&self) -> u64 {
        self.ops
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for LuserOp {
    fn code(&self) -> u16 {
        codes::RPL_LUSEROP
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<LuserOp> for Message {
    fn from(value: LuserOp) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<LuserOp> for RawMessage {
    fn from(value: LuserOp) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for LuserOp {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<LuserOp, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let ops = match p.as_str().parse::<u64>() {
            Ok(n) => n,
            Err(inner) => {
                return Err(ReplyError::Int {
                    string: String::from(p),
                    inner,
                })
            }
        };
        Ok(LuserOp {
            parameters,
            client,
            ops,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LuserUnknown {
    parameters: ParameterList,
    client: ReplyTarget,
    connections: u64,
}

impl LuserUnknown {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn connections(&self) -> u64 {
        self.connections
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for LuserUnknown {
    fn code(&self) -> u16 {
        codes::RPL_LUSERUNKNOWN
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<LuserUnknown> for Message {
    fn from(value: LuserUnknown) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<LuserUnknown> for RawMessage {
    fn from(value: LuserUnknown) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for LuserUnknown {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<LuserUnknown, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let connections = match p.as_str().parse::<u64>() {
            Ok(n) => n,
            Err(inner) => {
                return Err(ReplyError::Int {
                    string: String::from(p),
                    inner,
                })
            }
        };
        Ok(LuserUnknown {
            parameters,
            client,
            connections,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LuserChannels {
    parameters: ParameterList,
    client: ReplyTarget,
    channels: u64,
}

impl LuserChannels {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channels(&self) -> u64 {
        self.channels
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for LuserChannels {
    fn code(&self) -> u16 {
        codes::RPL_LUSERCHANNELS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<LuserChannels> for Message {
    fn from(value: LuserChannels) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<LuserChannels> for RawMessage {
    fn from(value: LuserChannels) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for LuserChannels {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<LuserChannels, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channels = match p.as_str().parse::<u64>() {
            Ok(n) => n,
            Err(inner) => {
                return Err(ReplyError::Int {
                    string: String::from(p),
                    inner,
                })
            }
        };
        Ok(LuserChannels {
            parameters,
            client,
            channels,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LuserMe {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl LuserMe {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for LuserMe {
    fn code(&self) -> u16 {
        codes::RPL_LUSERME
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<LuserMe> for Message {
    fn from(value: LuserMe) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<LuserMe> for RawMessage {
    fn from(value: LuserMe) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for LuserMe {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<LuserMe, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(LuserMe { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminMe {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl AdminMe {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn server(&self) -> Option<&str> {
        self.parameters.get(1).map(|p| p.as_str())
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for AdminMe {
    fn code(&self) -> u16 {
        codes::RPL_ADMINME
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<AdminMe> for Message {
    fn from(value: AdminMe) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<AdminMe> for RawMessage {
    fn from(value: AdminMe) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for AdminMe {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<AdminMe, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(AdminMe { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminLoc1 {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl AdminLoc1 {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for AdminLoc1 {
    fn code(&self) -> u16 {
        codes::RPL_ADMINLOC1
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<AdminLoc1> for Message {
    fn from(value: AdminLoc1) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<AdminLoc1> for RawMessage {
    fn from(value: AdminLoc1) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for AdminLoc1 {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<AdminLoc1, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(AdminLoc1 { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminLoc2 {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl AdminLoc2 {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for AdminLoc2 {
    fn code(&self) -> u16 {
        codes::RPL_ADMINLOC2
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<AdminLoc2> for Message {
    fn from(value: AdminLoc2) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<AdminLoc2> for RawMessage {
    fn from(value: AdminLoc2) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for AdminLoc2 {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<AdminLoc2, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(AdminLoc2 { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminEmail {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl AdminEmail {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for AdminEmail {
    fn code(&self) -> u16 {
        codes::RPL_ADMINEMAIL
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<AdminEmail> for Message {
    fn from(value: AdminEmail) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<AdminEmail> for RawMessage {
    fn from(value: AdminEmail) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for AdminEmail {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<AdminEmail, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(AdminEmail { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TryAgain {
    parameters: ParameterList,
    client: ReplyTarget,
    command: Verb,
}

impl TryAgain {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn command(&self) -> &Verb {
        &self.command
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for TryAgain {
    fn code(&self) -> u16 {
        codes::RPL_TRYAGAIN
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<TryAgain> for Message {
    fn from(value: TryAgain) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<TryAgain> for RawMessage {
    fn from(value: TryAgain) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for TryAgain {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<TryAgain, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 2");
        let command = Verb::from(String::from(p));
        Ok(TryAgain {
            parameters,
            client,
            command,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalUsers {
    parameters: ParameterList,
    client: ReplyTarget,
    current_users: Option<u64>,
    max_users: Option<u64>,
}

impl LocalUsers {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn current_users(&self) -> Option<u64> {
        self.current_users
    }

    pub fn max_users(&self) -> Option<u64> {
        self.max_users
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for LocalUsers {
    fn code(&self) -> u16 {
        codes::RPL_LOCALUSERS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<LocalUsers> for Message {
    fn from(value: LocalUsers) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<LocalUsers> for RawMessage {
    fn from(value: LocalUsers) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for LocalUsers {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<LocalUsers, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        let current_users = (parameters.len() > 2)
            .then(|| {
                let p = parameters
                    .get(1)
                    .expect("Parameter 1 should exist when list length is at least 2");
                match p.as_str().parse::<u64>() {
                    Ok(n) => Ok(n),
                    Err(inner) => Err(ReplyError::Int {
                        string: String::from(p),
                        inner,
                    }),
                }
            })
            .transpose()?;
        let max_users = (parameters.len() > 3)
            .then(|| {
                let p = parameters
                    .get(2)
                    .expect("Parameter 2 should exist when list length is at least 2");
                match p.as_str().parse::<u64>() {
                    Ok(n) => Ok(n),
                    Err(inner) => Err(ReplyError::Int {
                        string: String::from(p),
                        inner,
                    }),
                }
            })
            .transpose()?;
        Ok(LocalUsers {
            parameters,
            client,
            current_users,
            max_users,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalUsers {
    parameters: ParameterList,
    client: ReplyTarget,
    current_users: Option<u64>,
    max_users: Option<u64>,
}

impl GlobalUsers {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn current_users(&self) -> Option<u64> {
        self.current_users
    }

    pub fn max_users(&self) -> Option<u64> {
        self.max_users
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for GlobalUsers {
    fn code(&self) -> u16 {
        codes::RPL_GLOBALUSERS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<GlobalUsers> for Message {
    fn from(value: GlobalUsers) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<GlobalUsers> for RawMessage {
    fn from(value: GlobalUsers) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for GlobalUsers {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<GlobalUsers, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        let current_users = (parameters.len() > 2)
            .then(|| {
                let p = parameters
                    .get(1)
                    .expect("Parameter 1 should exist when list length is at least 2");
                match p.as_str().parse::<u64>() {
                    Ok(n) => Ok(n),
                    Err(inner) => Err(ReplyError::Int {
                        string: String::from(p),
                        inner,
                    }),
                }
            })
            .transpose()?;
        let max_users = (parameters.len() > 3)
            .then(|| {
                let p = parameters
                    .get(2)
                    .expect("Parameter 2 should exist when list length is at least 2");
                match p.as_str().parse::<u64>() {
                    Ok(n) => Ok(n),
                    Err(inner) => Err(ReplyError::Int {
                        string: String::from(p),
                        inner,
                    }),
                }
            })
            .transpose()?;
        Ok(GlobalUsers {
            parameters,
            client,
            current_users,
            max_users,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIsCertFP {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
}

impl WhoIsCertFP {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for WhoIsCertFP {
    fn code(&self) -> u16 {
        codes::RPL_WHOISCERTFP
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WhoIsCertFP> for Message {
    fn from(value: WhoIsCertFP) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WhoIsCertFP> for RawMessage {
    fn from(value: WhoIsCertFP) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WhoIsCertFP {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WhoIsCertFP, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        Ok(WhoIsCertFP {
            parameters,
            client,
            nickname,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct None {
    parameters: ParameterList,
}

impl ReplyParts for None {
    fn code(&self) -> u16 {
        codes::RPL_NONE
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<None> for Message {
    fn from(value: None) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<None> for RawMessage {
    fn from(value: None) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for None {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<None, ReplyError> {
        Ok(None { parameters })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Away {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
}

impl Away {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for Away {
    fn code(&self) -> u16 {
        codes::RPL_AWAY
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<Away> for Message {
    fn from(value: Away) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<Away> for RawMessage {
    fn from(value: Away) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for Away {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<Away, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        Ok(Away {
            parameters,
            client,
            nickname,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserHostRpl {
    parameters: ParameterList,
    client: ReplyTarget,
    replies: Vec<UserHostReply>,
}

impl UserHostRpl {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn replies(&self) -> &[UserHostReply] {
        &self.replies
    }
}

impl ReplyParts for UserHostRpl {
    fn code(&self) -> u16 {
        codes::RPL_USERHOST
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<UserHostRpl> for Message {
    fn from(value: UserHostRpl) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<UserHostRpl> for RawMessage {
    fn from(value: UserHostRpl) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for UserHostRpl {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<UserHostRpl, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .last()
            .expect("Parameter list should be nonempty when list length is at least 2");
        let replies = split_spaces(p.as_str())
            .map(|s| UserHostReply::try_from(s.to_owned()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(UserHostRpl {
            parameters,
            client,
            replies,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnAway {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl UnAway {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for UnAway {
    fn code(&self) -> u16 {
        codes::RPL_UNAWAY
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<UnAway> for Message {
    fn from(value: UnAway) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<UnAway> for RawMessage {
    fn from(value: UnAway) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for UnAway {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<UnAway, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(UnAway { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NowAway {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl NowAway {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NowAway {
    fn code(&self) -> u16 {
        codes::RPL_NOWAWAY
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NowAway> for Message {
    fn from(value: NowAway) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NowAway> for RawMessage {
    fn from(value: NowAway) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NowAway {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NowAway, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(NowAway { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIsRegNick {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
}

impl WhoIsRegNick {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for WhoIsRegNick {
    fn code(&self) -> u16 {
        codes::RPL_WHOISREGNICK
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WhoIsRegNick> for Message {
    fn from(value: WhoIsRegNick) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WhoIsRegNick> for RawMessage {
    fn from(value: WhoIsRegNick) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WhoIsRegNick {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WhoIsRegNick, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        Ok(WhoIsRegNick {
            parameters,
            client,
            nickname,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIsUser {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
    username: Username,
}

impl WhoIsUser {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn host(&self) -> &str {
        let Some(p) = self.parameters.get(3) else {
            unreachable!("index 3 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn realname(&self) -> &str {
        let Some(p) = self.parameters.get(5) else {
            unreachable!("index 5 should exist in reply parameters");
        };
        p.as_str()
    }
}

impl ReplyParts for WhoIsUser {
    fn code(&self) -> u16 {
        codes::RPL_WHOISUSER
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WhoIsUser> for Message {
    fn from(value: WhoIsUser) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WhoIsUser> for RawMessage {
    fn from(value: WhoIsUser) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WhoIsUser {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WhoIsUser, ReplyError> {
        if parameters.len() < 6 {
            return Err(ReplyError::ParamQty {
                min_required: 6,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 6");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 6");
        let nickname = Nickname::try_from(String::from(p))?;
        let p = parameters
            .get(2)
            .expect("Parameter 2 should exist when list length is at least 6");
        let username = Username::try_from(String::from(p))?;
        Ok(WhoIsUser {
            parameters,
            client,
            nickname,
            username,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIsServer {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
}

impl WhoIsServer {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn server(&self) -> &str {
        let Some(p) = self.parameters.get(2) else {
            unreachable!("index 2 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn server_info(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for WhoIsServer {
    fn code(&self) -> u16 {
        codes::RPL_WHOISSERVER
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WhoIsServer> for Message {
    fn from(value: WhoIsServer) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WhoIsServer> for RawMessage {
    fn from(value: WhoIsServer) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WhoIsServer {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WhoIsServer, ReplyError> {
        if parameters.len() < 4 {
            return Err(ReplyError::ParamQty {
                min_required: 4,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 4");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 4");
        let nickname = Nickname::try_from(String::from(p))?;
        Ok(WhoIsServer {
            parameters,
            client,
            nickname,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIsOperator {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
}

impl WhoIsOperator {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for WhoIsOperator {
    fn code(&self) -> u16 {
        codes::RPL_WHOISOPERATOR
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WhoIsOperator> for Message {
    fn from(value: WhoIsOperator) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WhoIsOperator> for RawMessage {
    fn from(value: WhoIsOperator) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WhoIsOperator {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WhoIsOperator, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        Ok(WhoIsOperator {
            parameters,
            client,
            nickname,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoWasUser {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
    username: Username,
}

impl WhoWasUser {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn host(&self) -> &str {
        let Some(p) = self.parameters.get(3) else {
            unreachable!("index 3 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn realname(&self) -> &str {
        let Some(p) = self.parameters.get(5) else {
            unreachable!("index 5 should exist in reply parameters");
        };
        p.as_str()
    }
}

impl ReplyParts for WhoWasUser {
    fn code(&self) -> u16 {
        codes::RPL_WHOWASUSER
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WhoWasUser> for Message {
    fn from(value: WhoWasUser) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WhoWasUser> for RawMessage {
    fn from(value: WhoWasUser) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WhoWasUser {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WhoWasUser, ReplyError> {
        if parameters.len() < 6 {
            return Err(ReplyError::ParamQty {
                min_required: 6,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 6");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 6");
        let nickname = Nickname::try_from(String::from(p))?;
        let p = parameters
            .get(2)
            .expect("Parameter 2 should exist when list length is at least 6");
        let username = Username::try_from(String::from(p))?;
        Ok(WhoWasUser {
            parameters,
            client,
            nickname,
            username,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndOfWho {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl EndOfWho {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn mask(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for EndOfWho {
    fn code(&self) -> u16 {
        codes::RPL_ENDOFWHO
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<EndOfWho> for Message {
    fn from(value: EndOfWho) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<EndOfWho> for RawMessage {
    fn from(value: EndOfWho) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for EndOfWho {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<EndOfWho, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(EndOfWho { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIsIdle {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
    secs: u64,
    signon: u64,
}

impl WhoIsIdle {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn secs(&self) -> u64 {
        self.secs
    }

    pub fn signon(&self) -> u64 {
        self.signon
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for WhoIsIdle {
    fn code(&self) -> u16 {
        codes::RPL_WHOISIDLE
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WhoIsIdle> for Message {
    fn from(value: WhoIsIdle) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WhoIsIdle> for RawMessage {
    fn from(value: WhoIsIdle) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WhoIsIdle {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WhoIsIdle, ReplyError> {
        if parameters.len() < 5 {
            return Err(ReplyError::ParamQty {
                min_required: 5,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 5");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 5");
        let nickname = Nickname::try_from(String::from(p))?;
        let p = parameters
            .get(2)
            .expect("Parameter 2 should exist when list length is at least 5");
        let secs = match p.as_str().parse::<u64>() {
            Ok(n) => n,
            Err(inner) => {
                return Err(ReplyError::Int {
                    string: String::from(p),
                    inner,
                })
            }
        };
        let p = parameters
            .get(3)
            .expect("Parameter 3 should exist when list length is at least 5");
        let signon = match p.as_str().parse::<u64>() {
            Ok(n) => n,
            Err(inner) => {
                return Err(ReplyError::Int {
                    string: String::from(p),
                    inner,
                })
            }
        };
        Ok(WhoIsIdle {
            parameters,
            client,
            nickname,
            secs,
            signon,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndOfWhoIs {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
}

impl EndOfWhoIs {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for EndOfWhoIs {
    fn code(&self) -> u16 {
        codes::RPL_ENDOFWHOIS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<EndOfWhoIs> for Message {
    fn from(value: EndOfWhoIs) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<EndOfWhoIs> for RawMessage {
    fn from(value: EndOfWhoIs) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for EndOfWhoIs {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<EndOfWhoIs, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        Ok(EndOfWhoIs {
            parameters,
            client,
            nickname,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIsChannels {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
    channels: Vec<(Option<char>, Channel)>,
}

impl WhoIsChannels {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn channels(&self) -> &[(Option<char>, Channel)] {
        &self.channels
    }
}

impl ReplyParts for WhoIsChannels {
    fn code(&self) -> u16 {
        codes::RPL_WHOISCHANNELS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WhoIsChannels> for Message {
    fn from(value: WhoIsChannels) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WhoIsChannels> for RawMessage {
    fn from(value: WhoIsChannels) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WhoIsChannels {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WhoIsChannels, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        let p = parameters
            .last()
            .expect("Parameter list should be nonempty when list length is at least 3");
        let channels = split_spaces(p.as_str())
            .map(|s| {
                let (prefix, s) = pop_channel_membership(s);
                Channel::try_from(s.to_owned()).map(|chan| (prefix, chan))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(WhoIsChannels {
            parameters,
            client,
            nickname,
            channels,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIsSpecial {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
}

impl WhoIsSpecial {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for WhoIsSpecial {
    fn code(&self) -> u16 {
        codes::RPL_WHOISSPECIAL
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WhoIsSpecial> for Message {
    fn from(value: WhoIsSpecial) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WhoIsSpecial> for RawMessage {
    fn from(value: WhoIsSpecial) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WhoIsSpecial {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WhoIsSpecial, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        Ok(WhoIsSpecial {
            parameters,
            client,
            nickname,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListStart {
    parameters: ParameterList,
}

impl ListStart {
    pub fn client(&self) -> &str {
        let Some(p) = self.parameters.get(0) else {
            unreachable!("index 0 should exist in reply parameters");
        };
        p.as_str()
    }
}

impl ReplyParts for ListStart {
    fn code(&self) -> u16 {
        codes::RPL_LISTSTART
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<ListStart> for Message {
    fn from(value: ListStart) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<ListStart> for RawMessage {
    fn from(value: ListStart) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for ListStart {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<ListStart, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        Ok(ListStart { parameters })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct List {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
    clients: u64,
}

impl List {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn clients(&self) -> u64 {
        self.clients
    }

    pub fn topic(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for List {
    fn code(&self) -> u16 {
        codes::RPL_LIST
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<List> for Message {
    fn from(value: List) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<List> for RawMessage {
    fn from(value: List) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for List {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<List, ReplyError> {
        if parameters.len() < 4 {
            return Err(ReplyError::ParamQty {
                min_required: 4,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 4");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 4");
        let channel = Channel::try_from(String::from(p))?;
        let p = parameters
            .get(2)
            .expect("Parameter 2 should exist when list length is at least 4");
        let clients = match p.as_str().parse::<u64>() {
            Ok(n) => n,
            Err(inner) => {
                return Err(ReplyError::Int {
                    string: String::from(p),
                    inner,
                })
            }
        };
        Ok(List {
            parameters,
            client,
            channel,
            clients,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListEnd {
    parameters: ParameterList,
}

impl ListEnd {
    pub fn client(&self) -> &str {
        let Some(p) = self.parameters.get(0) else {
            unreachable!("index 0 should exist in reply parameters");
        };
        p.as_str()
    }
}

impl ReplyParts for ListEnd {
    fn code(&self) -> u16 {
        codes::RPL_LISTEND
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<ListEnd> for Message {
    fn from(value: ListEnd) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<ListEnd> for RawMessage {
    fn from(value: ListEnd) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for ListEnd {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<ListEnd, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        Ok(ListEnd { parameters })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelModeIs {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
    modestring: ModeString,
    arguments: ParameterList,
}

impl ChannelModeIs {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn modestring(&self) -> &ModeString {
        &self.modestring
    }

    pub fn arguments(&self) -> &ParameterList {
        &self.arguments
    }
}

impl ReplyParts for ChannelModeIs {
    fn code(&self) -> u16 {
        codes::RPL_CHANNELMODEIS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<ChannelModeIs> for Message {
    fn from(value: ChannelModeIs) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<ChannelModeIs> for RawMessage {
    fn from(value: ChannelModeIs) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for ChannelModeIs {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<ChannelModeIs, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        let p = parameters
            .get(2)
            .expect("Parameter 2 should exist when list length is at least 3");
        let modestring = ModeString::try_from(String::from(p))?;
        let mut iter = parameters.clone().into_iter();
        for _ in 0..2 {
            let _ = iter.next();
        }
        let arguments = iter.into_parameter_list();
        Ok(ChannelModeIs {
            parameters,
            client,
            channel,
            modestring,
            arguments,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreationTime {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
    creationtime: u64,
}

impl CreationTime {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn creationtime(&self) -> u64 {
        self.creationtime
    }
}

impl ReplyParts for CreationTime {
    fn code(&self) -> u16 {
        codes::RPL_CREATIONTIME
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<CreationTime> for Message {
    fn from(value: CreationTime) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<CreationTime> for RawMessage {
    fn from(value: CreationTime) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for CreationTime {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<CreationTime, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        let p = parameters
            .get(2)
            .expect("Parameter 2 should exist when list length is at least 3");
        let creationtime = match p.as_str().parse::<u64>() {
            Ok(n) => n,
            Err(inner) => {
                return Err(ReplyError::Int {
                    string: String::from(p),
                    inner,
                })
            }
        };
        Ok(CreationTime {
            parameters,
            client,
            channel,
            creationtime,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIsAccount {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
}

impl WhoIsAccount {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn account(&self) -> &str {
        let Some(p) = self.parameters.get(2) else {
            unreachable!("index 2 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for WhoIsAccount {
    fn code(&self) -> u16 {
        codes::RPL_WHOISACCOUNT
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WhoIsAccount> for Message {
    fn from(value: WhoIsAccount) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WhoIsAccount> for RawMessage {
    fn from(value: WhoIsAccount) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WhoIsAccount {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WhoIsAccount, ReplyError> {
        if parameters.len() < 4 {
            return Err(ReplyError::ParamQty {
                min_required: 4,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 4");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 4");
        let nickname = Nickname::try_from(String::from(p))?;
        Ok(WhoIsAccount {
            parameters,
            client,
            nickname,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoTopic {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl NoTopic {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NoTopic {
    fn code(&self) -> u16 {
        codes::RPL_NOTOPIC
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NoTopic> for Message {
    fn from(value: NoTopic) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NoTopic> for RawMessage {
    fn from(value: NoTopic) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NoTopic {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NoTopic, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(NoTopic {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Topic {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl Topic {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn topic(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for Topic {
    fn code(&self) -> u16 {
        codes::RPL_TOPIC
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<Topic> for Message {
    fn from(value: Topic) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<Topic> for RawMessage {
    fn from(value: Topic) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for Topic {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<Topic, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(Topic {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopicWhoTime {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
    nickname: Nickname,
    setat: u64,
}

impl TopicWhoTime {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn setat(&self) -> u64 {
        self.setat
    }
}

impl ReplyParts for TopicWhoTime {
    fn code(&self) -> u16 {
        codes::RPL_TOPICWHOTIME
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<TopicWhoTime> for Message {
    fn from(value: TopicWhoTime) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<TopicWhoTime> for RawMessage {
    fn from(value: TopicWhoTime) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for TopicWhoTime {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<TopicWhoTime, ReplyError> {
        if parameters.len() < 4 {
            return Err(ReplyError::ParamQty {
                min_required: 4,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 4");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 4");
        let channel = Channel::try_from(String::from(p))?;
        let p = parameters
            .get(2)
            .expect("Parameter 2 should exist when list length is at least 4");
        let nickname = Nickname::try_from(String::from(p))?;
        let p = parameters
            .get(3)
            .expect("Parameter 3 should exist when list length is at least 4");
        let setat = match p.as_str().parse::<u64>() {
            Ok(n) => n,
            Err(inner) => {
                return Err(ReplyError::Int {
                    string: String::from(p),
                    inner,
                })
            }
        };
        Ok(TopicWhoTime {
            parameters,
            client,
            channel,
            nickname,
            setat,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InviteList {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl InviteList {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }
}

impl ReplyParts for InviteList {
    fn code(&self) -> u16 {
        codes::RPL_INVITELIST
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<InviteList> for Message {
    fn from(value: InviteList) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<InviteList> for RawMessage {
    fn from(value: InviteList) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for InviteList {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<InviteList, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 2");
        let channel = Channel::try_from(String::from(p))?;
        Ok(InviteList {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndOfInviteList {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl EndOfInviteList {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for EndOfInviteList {
    fn code(&self) -> u16 {
        codes::RPL_ENDOFINVITELIST
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<EndOfInviteList> for Message {
    fn from(value: EndOfInviteList) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<EndOfInviteList> for RawMessage {
    fn from(value: EndOfInviteList) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for EndOfInviteList {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<EndOfInviteList, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(EndOfInviteList { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIsActually {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
    host: Option<Host>,
    username: Option<Username>,
    ip: Option<IpAddr>,
}

impl WhoIsActually {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn host(&self) -> Option<&Host> {
        self.host.as_ref()
    }

    pub fn username(&self) -> Option<&Username> {
        self.username.as_ref()
    }

    pub fn ip(&self) -> Option<IpAddr> {
        self.ip
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for WhoIsActually {
    fn code(&self) -> u16 {
        codes::RPL_WHOISACTUALLY
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WhoIsActually> for Message {
    fn from(value: WhoIsActually) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WhoIsActually> for RawMessage {
    fn from(value: WhoIsActually) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WhoIsActually {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WhoIsActually, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        let (username, host, ip) = match parameters.len() {
            3 => (None, None, None),
            4 => {
                let p = parameters
                    .get(2)
                    .expect("Parameter 2 should exist when list length is at least 3");
                match Host::parse(p.as_str()) {
                    Ok(host @ Host::Domain(_)) => (None, Some(host), None),
                    Ok(Host::Ipv4(ip)) => (None, None, Some(IpAddr::from(ip))),
                    Ok(Host::Ipv6(ip)) => (None, None, Some(IpAddr::from(ip))),
                    Err(inner) => {
                        return Err(ReplyError::Host {
                            inner,
                            string: String::from(p),
                        })
                    }
                }
            }
            _ => {
                let p = parameters
                    .get(2)
                    .expect("Parameter 2 should exist when list length is at least 5");
                let (username, host) = if let Some((user, host)) = p.as_str().rsplit_once('@') {
                    let user = Username::try_from(user.to_owned())?;
                    let host = match Host::parse(host) {
                        Ok(host) => host,
                        Err(inner) => {
                            return Err(ReplyError::Host {
                                inner,
                                string: host.to_owned(),
                            })
                        }
                    };
                    (user, host)
                } else {
                    return Err(ReplyError::NoAt(String::from(p)));
                };
                let p = parameters
                    .get(3)
                    .expect("Parameter 3 should exist when list length is at least 5");
                let ip = match p.as_str().parse::<IpAddr>() {
                    Ok(ip) => ip,
                    Err(inner) => {
                        return Err(ReplyError::IpAddr {
                            inner,
                            string: String::from(p),
                        })
                    }
                };
                (Some(username), Some(host), Some(ip))
            }
        };
        Ok(WhoIsActually {
            parameters,
            client,
            nickname,
            host,
            username,
            ip,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Inviting {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
    channel: Channel,
}

impl Inviting {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }
}

impl ReplyParts for Inviting {
    fn code(&self) -> u16 {
        codes::RPL_INVITING
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<Inviting> for Message {
    fn from(value: Inviting) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<Inviting> for RawMessage {
    fn from(value: Inviting) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for Inviting {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<Inviting, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        let p = parameters
            .get(2)
            .expect("Parameter 2 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(Inviting {
            parameters,
            client,
            nickname,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvExList {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl InvExList {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn mask(&self) -> &str {
        let Some(p) = self.parameters.get(2) else {
            unreachable!("index 2 should exist in reply parameters");
        };
        p.as_str()
    }
}

impl ReplyParts for InvExList {
    fn code(&self) -> u16 {
        codes::RPL_INVEXLIST
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<InvExList> for Message {
    fn from(value: InvExList) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<InvExList> for RawMessage {
    fn from(value: InvExList) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for InvExList {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<InvExList, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(InvExList {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndOfInvExList {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl EndOfInvExList {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for EndOfInvExList {
    fn code(&self) -> u16 {
        codes::RPL_ENDOFINVEXLIST
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<EndOfInvExList> for Message {
    fn from(value: EndOfInvExList) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<EndOfInvExList> for RawMessage {
    fn from(value: EndOfInvExList) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for EndOfInvExList {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<EndOfInvExList, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(EndOfInvExList {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExceptList {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl ExceptList {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn mask(&self) -> &str {
        let Some(p) = self.parameters.get(2) else {
            unreachable!("index 2 should exist in reply parameters");
        };
        p.as_str()
    }
}

impl ReplyParts for ExceptList {
    fn code(&self) -> u16 {
        codes::RPL_EXCEPTLIST
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<ExceptList> for Message {
    fn from(value: ExceptList) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<ExceptList> for RawMessage {
    fn from(value: ExceptList) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for ExceptList {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<ExceptList, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(ExceptList {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndOfExceptList {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl EndOfExceptList {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for EndOfExceptList {
    fn code(&self) -> u16 {
        codes::RPL_ENDOFEXCEPTLIST
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<EndOfExceptList> for Message {
    fn from(value: EndOfExceptList) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<EndOfExceptList> for RawMessage {
    fn from(value: EndOfExceptList) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for EndOfExceptList {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<EndOfExceptList, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(EndOfExceptList {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Version {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl Version {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn version(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn server(&self) -> &str {
        let Some(p) = self.parameters.get(2) else {
            unreachable!("index 2 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn comments(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for Version {
    fn code(&self) -> u16 {
        codes::RPL_VERSION
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<Version> for Message {
    fn from(value: Version) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<Version> for RawMessage {
    fn from(value: Version) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for Version {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<Version, ReplyError> {
        if parameters.len() < 4 {
            return Err(ReplyError::ParamQty {
                min_required: 4,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 4");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(Version { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoReply {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
    username: Username,
    nickname: Nickname,
    flags: WhoFlags,
    hopcount: u32,
}

impl WhoReply {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn host(&self) -> &str {
        let Some(p) = self.parameters.get(3) else {
            unreachable!("index 3 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn server(&self) -> &str {
        let Some(p) = self.parameters.get(4) else {
            unreachable!("index 4 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn flags(&self) -> &WhoFlags {
        &self.flags
    }

    pub fn hopcount(&self) -> u32 {
        self.hopcount
    }

    pub fn realname(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        split_word(p.as_str()).1
    }
}

impl ReplyParts for WhoReply {
    fn code(&self) -> u16 {
        codes::RPL_WHOREPLY
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WhoReply> for Message {
    fn from(value: WhoReply) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WhoReply> for RawMessage {
    fn from(value: WhoReply) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WhoReply {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WhoReply, ReplyError> {
        if parameters.len() < 8 {
            return Err(ReplyError::ParamQty {
                min_required: 8,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 8");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 8");
        let channel = Channel::try_from(String::from(p))?;
        let p = parameters
            .get(2)
            .expect("Parameter 2 should exist when list length is at least 8");
        let username = Username::try_from(String::from(p))?;
        let p = parameters
            .get(5)
            .expect("Parameter 5 should exist when list length is at least 8");
        let nickname = Nickname::try_from(String::from(p))?;
        let p = parameters
            .get(6)
            .expect("Parameter 6 should exist when list length is at least 8");
        let flags = WhoFlags::try_from(String::from(p))?;
        let p = parameters
            .last()
            .expect("Parameter list should be nonempty when list length is at least 8");
        let word = split_word(p.as_str()).0;
        let hopcount = match word.parse::<u32>() {
            Ok(n) => n,
            Err(inner) => {
                return Err(ReplyError::Int {
                    string: String::from(word),
                    inner,
                })
            }
        };
        Ok(WhoReply {
            parameters,
            client,
            channel,
            username,
            nickname,
            flags,
            hopcount,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamReply {
    parameters: ParameterList,
    client: ReplyTarget,
    channel_status: ChannelStatus,
    channel: Channel,
    clients: Vec<(Option<char>, Nickname)>,
}

impl NamReply {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel_status(&self) -> &ChannelStatus {
        &self.channel_status
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn clients(&self) -> &[(Option<char>, Nickname)] {
        &self.clients
    }
}

impl ReplyParts for NamReply {
    fn code(&self) -> u16 {
        codes::RPL_NAMREPLY
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NamReply> for Message {
    fn from(value: NamReply) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NamReply> for RawMessage {
    fn from(value: NamReply) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NamReply {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NamReply, ReplyError> {
        if parameters.len() < 4 {
            return Err(ReplyError::ParamQty {
                min_required: 4,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 4");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 4");
        let channel_status = ChannelStatus::try_from(String::from(p))?;
        let p = parameters
            .get(2)
            .expect("Parameter 2 should exist when list length is at least 4");
        let channel = Channel::try_from(String::from(p))?;
        let p = parameters
            .last()
            .expect("Parameter list should be nonempty when list length is at least 4");
        let clients = split_spaces(p.as_str())
            .map(|s| {
                let (prefix, s) = pop_channel_membership(s);
                Nickname::try_from(s.to_owned()).map(|nick| (prefix, nick))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(NamReply {
            parameters,
            client,
            channel_status,
            channel,
            clients,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Links {
    parameters: ParameterList,
    client: ReplyTarget,
    hopcount: u32,
}

impl Links {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn server1(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn server2(&self) -> &str {
        let Some(p) = self.parameters.get(2) else {
            unreachable!("index 2 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn hopcount(&self) -> u32 {
        self.hopcount
    }

    pub fn server_info(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        split_word(p.as_str()).1
    }
}

impl ReplyParts for Links {
    fn code(&self) -> u16 {
        codes::RPL_LINKS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<Links> for Message {
    fn from(value: Links) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<Links> for RawMessage {
    fn from(value: Links) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for Links {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<Links, ReplyError> {
        if parameters.len() < 4 {
            return Err(ReplyError::ParamQty {
                min_required: 4,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 4");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .last()
            .expect("Parameter list should be nonempty when list length is at least 4");
        let word = split_word(p.as_str()).0;
        let hopcount = match word.parse::<u32>() {
            Ok(n) => n,
            Err(inner) => {
                return Err(ReplyError::Int {
                    string: String::from(word),
                    inner,
                })
            }
        };
        Ok(Links {
            parameters,
            client,
            hopcount,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndOfLinks {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl EndOfLinks {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for EndOfLinks {
    fn code(&self) -> u16 {
        codes::RPL_ENDOFLINKS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<EndOfLinks> for Message {
    fn from(value: EndOfLinks) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<EndOfLinks> for RawMessage {
    fn from(value: EndOfLinks) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for EndOfLinks {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<EndOfLinks, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(EndOfLinks { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndOfNames {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl EndOfNames {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for EndOfNames {
    fn code(&self) -> u16 {
        codes::RPL_ENDOFNAMES
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<EndOfNames> for Message {
    fn from(value: EndOfNames) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<EndOfNames> for RawMessage {
    fn from(value: EndOfNames) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for EndOfNames {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<EndOfNames, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(EndOfNames {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BanList {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
    set_ts: Option<u64>,
}

impl BanList {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn mask(&self) -> &str {
        let Some(p) = self.parameters.get(2) else {
            unreachable!("index 2 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn who(&self) -> Option<&str> {
        self.parameters.get(3).map(|p| p.as_str())
    }

    pub fn set_ts(&self) -> Option<u64> {
        self.set_ts
    }
}

impl ReplyParts for BanList {
    fn code(&self) -> u16 {
        codes::RPL_BANLIST
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<BanList> for Message {
    fn from(value: BanList) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<BanList> for RawMessage {
    fn from(value: BanList) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for BanList {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<BanList, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        let set_ts = parameters
            .get(4)
            .map(|p| match p.as_str().parse::<u64>() {
                Ok(n) => Ok(n),
                Err(inner) => Err(ReplyError::Int {
                    string: String::from(p),
                    inner,
                }),
            })
            .transpose()?;
        Ok(BanList {
            parameters,
            client,
            channel,
            set_ts,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndOfBanList {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl EndOfBanList {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for EndOfBanList {
    fn code(&self) -> u16 {
        codes::RPL_ENDOFBANLIST
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<EndOfBanList> for Message {
    fn from(value: EndOfBanList) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<EndOfBanList> for RawMessage {
    fn from(value: EndOfBanList) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for EndOfBanList {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<EndOfBanList, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(EndOfBanList {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndOfWhoWas {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
}

impl EndOfWhoWas {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for EndOfWhoWas {
    fn code(&self) -> u16 {
        codes::RPL_ENDOFWHOWAS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<EndOfWhoWas> for Message {
    fn from(value: EndOfWhoWas) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<EndOfWhoWas> for RawMessage {
    fn from(value: EndOfWhoWas) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for EndOfWhoWas {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<EndOfWhoWas, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        Ok(EndOfWhoWas {
            parameters,
            client,
            nickname,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Info {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl Info {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for Info {
    fn code(&self) -> u16 {
        codes::RPL_INFO
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<Info> for Message {
    fn from(value: Info) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<Info> for RawMessage {
    fn from(value: Info) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for Info {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<Info, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(Info { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Motd {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl Motd {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for Motd {
    fn code(&self) -> u16 {
        codes::RPL_MOTD
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<Motd> for Message {
    fn from(value: Motd) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<Motd> for RawMessage {
    fn from(value: Motd) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for Motd {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<Motd, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(Motd { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndOfInfo {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl EndOfInfo {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for EndOfInfo {
    fn code(&self) -> u16 {
        codes::RPL_ENDOFINFO
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<EndOfInfo> for Message {
    fn from(value: EndOfInfo) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<EndOfInfo> for RawMessage {
    fn from(value: EndOfInfo) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for EndOfInfo {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<EndOfInfo, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(EndOfInfo { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotdStart {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl MotdStart {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for MotdStart {
    fn code(&self) -> u16 {
        codes::RPL_MOTDSTART
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<MotdStart> for Message {
    fn from(value: MotdStart) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<MotdStart> for RawMessage {
    fn from(value: MotdStart) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for MotdStart {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<MotdStart, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(MotdStart { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndOfMotd {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl EndOfMotd {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for EndOfMotd {
    fn code(&self) -> u16 {
        codes::RPL_ENDOFMOTD
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<EndOfMotd> for Message {
    fn from(value: EndOfMotd) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<EndOfMotd> for RawMessage {
    fn from(value: EndOfMotd) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for EndOfMotd {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<EndOfMotd, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(EndOfMotd { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIsHost {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
}

impl WhoIsHost {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for WhoIsHost {
    fn code(&self) -> u16 {
        codes::RPL_WHOISHOST
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WhoIsHost> for Message {
    fn from(value: WhoIsHost) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WhoIsHost> for RawMessage {
    fn from(value: WhoIsHost) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WhoIsHost {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WhoIsHost, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        Ok(WhoIsHost {
            parameters,
            client,
            nickname,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIsModes {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
}

impl WhoIsModes {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for WhoIsModes {
    fn code(&self) -> u16 {
        codes::RPL_WHOISMODES
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WhoIsModes> for Message {
    fn from(value: WhoIsModes) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WhoIsModes> for RawMessage {
    fn from(value: WhoIsModes) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WhoIsModes {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WhoIsModes, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        Ok(WhoIsModes {
            parameters,
            client,
            nickname,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YoureOper {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl YoureOper {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for YoureOper {
    fn code(&self) -> u16 {
        codes::RPL_YOUREOPER
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<YoureOper> for Message {
    fn from(value: YoureOper) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<YoureOper> for RawMessage {
    fn from(value: YoureOper) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for YoureOper {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<YoureOper, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(YoureOper { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rehashing {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl Rehashing {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn config_file(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for Rehashing {
    fn code(&self) -> u16 {
        codes::RPL_REHASHING
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<Rehashing> for Message {
    fn from(value: Rehashing) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<Rehashing> for RawMessage {
    fn from(value: Rehashing) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for Rehashing {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<Rehashing, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(Rehashing { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Time {
    parameters: ParameterList,
    client: ReplyTarget,
    timestamp: Option<u64>,
}

impl Time {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn server(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn timestamp(&self) -> Option<u64> {
        self.timestamp
    }

    pub fn ts_offset(&self) -> Option<&str> {
        self.parameters.get(3).map(|p| p.as_str())
    }

    pub fn human_time(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for Time {
    fn code(&self) -> u16 {
        codes::RPL_TIME
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<Time> for Message {
    fn from(value: Time) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<Time> for RawMessage {
    fn from(value: Time) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for Time {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<Time, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let timestamp = (parameters.len() > 3)
            .then(|| {
                let p = parameters
                    .get(2)
                    .expect("Parameter 2 should exist when list length is at least 3");
                match p.as_str().parse::<u64>() {
                    Ok(n) => Ok(n),
                    Err(inner) => Err(ReplyError::Int {
                        string: String::from(p),
                        inner,
                    }),
                }
            })
            .transpose()?;
        Ok(Time {
            parameters,
            client,
            timestamp,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownError {
    parameters: ParameterList,
    client: ReplyTarget,
    command: Verb,
    subcommands: Vec<String>,
}

impl UnknownError {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn command(&self) -> &Verb {
        &self.command
    }

    pub fn subcommands(&self) -> &[String] {
        &self.subcommands
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for UnknownError {
    fn code(&self) -> u16 {
        codes::ERR_UNKNOWNERROR
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<UnknownError> for Message {
    fn from(value: UnknownError) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<UnknownError> for RawMessage {
    fn from(value: UnknownError) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for UnknownError {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<UnknownError, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let command = Verb::from(String::from(p));
        let subcommands = parameters
            .iter()
            .skip(2)
            .take(parameters.len() - 3)
            .map(String::from)
            .collect::<Vec<_>>();
        Ok(UnknownError {
            parameters,
            client,
            command,
            subcommands,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoSuchNick {
    parameters: ParameterList,
    client: ReplyTarget,
    target: MsgTarget,
}

impl NoSuchNick {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn target(&self) -> &MsgTarget {
        &self.target
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NoSuchNick {
    fn code(&self) -> u16 {
        codes::ERR_NOSUCHNICK
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NoSuchNick> for Message {
    fn from(value: NoSuchNick) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NoSuchNick> for RawMessage {
    fn from(value: NoSuchNick) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NoSuchNick {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NoSuchNick, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let target = MsgTarget::try_from(String::from(p))?;
        Ok(NoSuchNick {
            parameters,
            client,
            target,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoSuchServer {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl NoSuchServer {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn server(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NoSuchServer {
    fn code(&self) -> u16 {
        codes::ERR_NOSUCHSERVER
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NoSuchServer> for Message {
    fn from(value: NoSuchServer) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NoSuchServer> for RawMessage {
    fn from(value: NoSuchServer) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NoSuchServer {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NoSuchServer, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(NoSuchServer { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoSuchChannel {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl NoSuchChannel {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NoSuchChannel {
    fn code(&self) -> u16 {
        codes::ERR_NOSUCHCHANNEL
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NoSuchChannel> for Message {
    fn from(value: NoSuchChannel) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NoSuchChannel> for RawMessage {
    fn from(value: NoSuchChannel) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NoSuchChannel {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NoSuchChannel, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(NoSuchChannel {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CannotSendToChan {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl CannotSendToChan {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for CannotSendToChan {
    fn code(&self) -> u16 {
        codes::ERR_CANNOTSENDTOCHAN
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<CannotSendToChan> for Message {
    fn from(value: CannotSendToChan) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<CannotSendToChan> for RawMessage {
    fn from(value: CannotSendToChan) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for CannotSendToChan {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<CannotSendToChan, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(CannotSendToChan {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TooManyChannels {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl TooManyChannels {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for TooManyChannels {
    fn code(&self) -> u16 {
        codes::ERR_TOOMANYCHANNELS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<TooManyChannels> for Message {
    fn from(value: TooManyChannels) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<TooManyChannels> for RawMessage {
    fn from(value: TooManyChannels) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for TooManyChannels {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<TooManyChannels, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(TooManyChannels {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasNoSuchNick {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
}

impl WasNoSuchNick {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for WasNoSuchNick {
    fn code(&self) -> u16 {
        codes::ERR_WASNOSUCHNICK
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WasNoSuchNick> for Message {
    fn from(value: WasNoSuchNick) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WasNoSuchNick> for RawMessage {
    fn from(value: WasNoSuchNick) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WasNoSuchNick {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WasNoSuchNick, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        Ok(WasNoSuchNick {
            parameters,
            client,
            nickname,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoOrigin {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl NoOrigin {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NoOrigin {
    fn code(&self) -> u16 {
        codes::ERR_NOORIGIN
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NoOrigin> for Message {
    fn from(value: NoOrigin) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NoOrigin> for RawMessage {
    fn from(value: NoOrigin) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NoOrigin {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NoOrigin, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(NoOrigin { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoRecipient {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl NoRecipient {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NoRecipient {
    fn code(&self) -> u16 {
        codes::ERR_NORECIPIENT
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NoRecipient> for Message {
    fn from(value: NoRecipient) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NoRecipient> for RawMessage {
    fn from(value: NoRecipient) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NoRecipient {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NoRecipient, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(NoRecipient { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoTextToSend {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl NoTextToSend {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NoTextToSend {
    fn code(&self) -> u16 {
        codes::ERR_NOTEXTTOSEND
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NoTextToSend> for Message {
    fn from(value: NoTextToSend) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NoTextToSend> for RawMessage {
    fn from(value: NoTextToSend) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NoTextToSend {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NoTextToSend, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(NoTextToSend { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputTooLong {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl InputTooLong {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for InputTooLong {
    fn code(&self) -> u16 {
        codes::ERR_INPUTTOOLONG
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<InputTooLong> for Message {
    fn from(value: InputTooLong) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<InputTooLong> for RawMessage {
    fn from(value: InputTooLong) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for InputTooLong {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<InputTooLong, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(InputTooLong { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownCommand {
    parameters: ParameterList,
    client: ReplyTarget,
    command: Verb,
}

impl UnknownCommand {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn command(&self) -> &Verb {
        &self.command
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for UnknownCommand {
    fn code(&self) -> u16 {
        codes::ERR_UNKNOWNCOMMAND
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<UnknownCommand> for Message {
    fn from(value: UnknownCommand) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<UnknownCommand> for RawMessage {
    fn from(value: UnknownCommand) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for UnknownCommand {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<UnknownCommand, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let command = Verb::from(String::from(p));
        Ok(UnknownCommand {
            parameters,
            client,
            command,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoMotd {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl NoMotd {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NoMotd {
    fn code(&self) -> u16 {
        codes::ERR_NOMOTD
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NoMotd> for Message {
    fn from(value: NoMotd) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NoMotd> for RawMessage {
    fn from(value: NoMotd) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NoMotd {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NoMotd, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(NoMotd { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoNicknameGiven {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl NoNicknameGiven {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NoNicknameGiven {
    fn code(&self) -> u16 {
        codes::ERR_NONICKNAMEGIVEN
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NoNicknameGiven> for Message {
    fn from(value: NoNicknameGiven) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NoNicknameGiven> for RawMessage {
    fn from(value: NoNicknameGiven) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NoNicknameGiven {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NoNicknameGiven, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(NoNicknameGiven { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErroneousNickname {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl ErroneousNickname {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for ErroneousNickname {
    fn code(&self) -> u16 {
        codes::ERR_ERRONEUSNICKNAME
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<ErroneousNickname> for Message {
    fn from(value: ErroneousNickname) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<ErroneousNickname> for RawMessage {
    fn from(value: ErroneousNickname) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for ErroneousNickname {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<ErroneousNickname, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(ErroneousNickname { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NicknameInUse {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
}

impl NicknameInUse {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NicknameInUse {
    fn code(&self) -> u16 {
        codes::ERR_NICKNAMEINUSE
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NicknameInUse> for Message {
    fn from(value: NicknameInUse) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NicknameInUse> for RawMessage {
    fn from(value: NicknameInUse) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NicknameInUse {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NicknameInUse, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        Ok(NicknameInUse {
            parameters,
            client,
            nickname,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NickCollision {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
}

impl NickCollision {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NickCollision {
    fn code(&self) -> u16 {
        codes::ERR_NICKCOLLISION
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NickCollision> for Message {
    fn from(value: NickCollision) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NickCollision> for RawMessage {
    fn from(value: NickCollision) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NickCollision {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NickCollision, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        Ok(NickCollision {
            parameters,
            client,
            nickname,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserNotInChannel {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
    channel: Channel,
}

impl UserNotInChannel {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for UserNotInChannel {
    fn code(&self) -> u16 {
        codes::ERR_USERNOTINCHANNEL
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<UserNotInChannel> for Message {
    fn from(value: UserNotInChannel) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<UserNotInChannel> for RawMessage {
    fn from(value: UserNotInChannel) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for UserNotInChannel {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<UserNotInChannel, ReplyError> {
        if parameters.len() < 4 {
            return Err(ReplyError::ParamQty {
                min_required: 4,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 4");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 4");
        let nickname = Nickname::try_from(String::from(p))?;
        let p = parameters
            .get(2)
            .expect("Parameter 2 should exist when list length is at least 4");
        let channel = Channel::try_from(String::from(p))?;
        Ok(UserNotInChannel {
            parameters,
            client,
            nickname,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotOnChannel {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl NotOnChannel {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NotOnChannel {
    fn code(&self) -> u16 {
        codes::ERR_NOTONCHANNEL
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NotOnChannel> for Message {
    fn from(value: NotOnChannel) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NotOnChannel> for RawMessage {
    fn from(value: NotOnChannel) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NotOnChannel {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NotOnChannel, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(NotOnChannel {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserOnChannel {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
    channel: Channel,
}

impl UserOnChannel {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for UserOnChannel {
    fn code(&self) -> u16 {
        codes::ERR_USERONCHANNEL
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<UserOnChannel> for Message {
    fn from(value: UserOnChannel) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<UserOnChannel> for RawMessage {
    fn from(value: UserOnChannel) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for UserOnChannel {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<UserOnChannel, ReplyError> {
        if parameters.len() < 4 {
            return Err(ReplyError::ParamQty {
                min_required: 4,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 4");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 4");
        let nickname = Nickname::try_from(String::from(p))?;
        let p = parameters
            .get(2)
            .expect("Parameter 2 should exist when list length is at least 4");
        let channel = Channel::try_from(String::from(p))?;
        Ok(UserOnChannel {
            parameters,
            client,
            nickname,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotRegistered {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl NotRegistered {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NotRegistered {
    fn code(&self) -> u16 {
        codes::ERR_NOTREGISTERED
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NotRegistered> for Message {
    fn from(value: NotRegistered) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NotRegistered> for RawMessage {
    fn from(value: NotRegistered) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NotRegistered {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NotRegistered, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(NotRegistered { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NeedMoreParams {
    parameters: ParameterList,
    client: ReplyTarget,
    command: Verb,
}

impl NeedMoreParams {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn command(&self) -> &Verb {
        &self.command
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NeedMoreParams {
    fn code(&self) -> u16 {
        codes::ERR_NEEDMOREPARAMS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NeedMoreParams> for Message {
    fn from(value: NeedMoreParams) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NeedMoreParams> for RawMessage {
    fn from(value: NeedMoreParams) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NeedMoreParams {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NeedMoreParams, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let command = Verb::from(String::from(p));
        Ok(NeedMoreParams {
            parameters,
            client,
            command,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlreadyRegistered {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl AlreadyRegistered {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for AlreadyRegistered {
    fn code(&self) -> u16 {
        codes::ERR_ALREADYREGISTERED
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<AlreadyRegistered> for Message {
    fn from(value: AlreadyRegistered) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<AlreadyRegistered> for RawMessage {
    fn from(value: AlreadyRegistered) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for AlreadyRegistered {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<AlreadyRegistered, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(AlreadyRegistered { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PasswdMismatch {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl PasswdMismatch {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for PasswdMismatch {
    fn code(&self) -> u16 {
        codes::ERR_PASSWDMISMATCH
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<PasswdMismatch> for Message {
    fn from(value: PasswdMismatch) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<PasswdMismatch> for RawMessage {
    fn from(value: PasswdMismatch) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for PasswdMismatch {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<PasswdMismatch, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(PasswdMismatch { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YoureBannedCreep {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl YoureBannedCreep {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for YoureBannedCreep {
    fn code(&self) -> u16 {
        codes::ERR_YOUREBANNEDCREEP
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<YoureBannedCreep> for Message {
    fn from(value: YoureBannedCreep) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<YoureBannedCreep> for RawMessage {
    fn from(value: YoureBannedCreep) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for YoureBannedCreep {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<YoureBannedCreep, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(YoureBannedCreep { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelIsFull {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl ChannelIsFull {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for ChannelIsFull {
    fn code(&self) -> u16 {
        codes::ERR_CHANNELISFULL
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<ChannelIsFull> for Message {
    fn from(value: ChannelIsFull) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<ChannelIsFull> for RawMessage {
    fn from(value: ChannelIsFull) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for ChannelIsFull {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<ChannelIsFull, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(ChannelIsFull {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownMode {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl UnknownMode {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn modechar(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for UnknownMode {
    fn code(&self) -> u16 {
        codes::ERR_UNKNOWNMODE
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<UnknownMode> for Message {
    fn from(value: UnknownMode) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<UnknownMode> for RawMessage {
    fn from(value: UnknownMode) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for UnknownMode {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<UnknownMode, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(UnknownMode { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InviteOnlyChan {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl InviteOnlyChan {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for InviteOnlyChan {
    fn code(&self) -> u16 {
        codes::ERR_INVITEONLYCHAN
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<InviteOnlyChan> for Message {
    fn from(value: InviteOnlyChan) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<InviteOnlyChan> for RawMessage {
    fn from(value: InviteOnlyChan) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for InviteOnlyChan {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<InviteOnlyChan, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(InviteOnlyChan {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BannedFromChan {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl BannedFromChan {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for BannedFromChan {
    fn code(&self) -> u16 {
        codes::ERR_BANNEDFROMCHAN
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<BannedFromChan> for Message {
    fn from(value: BannedFromChan) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<BannedFromChan> for RawMessage {
    fn from(value: BannedFromChan) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for BannedFromChan {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<BannedFromChan, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(BannedFromChan {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BadChannelKey {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl BadChannelKey {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for BadChannelKey {
    fn code(&self) -> u16 {
        codes::ERR_BADCHANNELKEY
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<BadChannelKey> for Message {
    fn from(value: BadChannelKey) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<BadChannelKey> for RawMessage {
    fn from(value: BadChannelKey) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for BadChannelKey {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<BadChannelKey, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(BadChannelKey {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BadChanMask {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl BadChanMask {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for BadChanMask {
    fn code(&self) -> u16 {
        codes::ERR_BADCHANMASK
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<BadChanMask> for Message {
    fn from(value: BadChanMask) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<BadChanMask> for RawMessage {
    fn from(value: BadChanMask) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for BadChanMask {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<BadChanMask, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(BadChanMask { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoPrivileges {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl NoPrivileges {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NoPrivileges {
    fn code(&self) -> u16 {
        codes::ERR_NOPRIVILEGES
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NoPrivileges> for Message {
    fn from(value: NoPrivileges) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NoPrivileges> for RawMessage {
    fn from(value: NoPrivileges) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NoPrivileges {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NoPrivileges, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(NoPrivileges { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChanOPrivsNeeded {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl ChanOPrivsNeeded {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for ChanOPrivsNeeded {
    fn code(&self) -> u16 {
        codes::ERR_CHANOPRIVSNEEDED
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<ChanOPrivsNeeded> for Message {
    fn from(value: ChanOPrivsNeeded) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<ChanOPrivsNeeded> for RawMessage {
    fn from(value: ChanOPrivsNeeded) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for ChanOPrivsNeeded {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<ChanOPrivsNeeded, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(ChanOPrivsNeeded {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CantKillServer {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl CantKillServer {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for CantKillServer {
    fn code(&self) -> u16 {
        codes::ERR_CANTKILLSERVER
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<CantKillServer> for Message {
    fn from(value: CantKillServer) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<CantKillServer> for RawMessage {
    fn from(value: CantKillServer) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for CantKillServer {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<CantKillServer, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(CantKillServer { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoOperHost {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl NoOperHost {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NoOperHost {
    fn code(&self) -> u16 {
        codes::ERR_NOOPERHOST
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NoOperHost> for Message {
    fn from(value: NoOperHost) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NoOperHost> for RawMessage {
    fn from(value: NoOperHost) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NoOperHost {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NoOperHost, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(NoOperHost { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UmodeUnknownFlag {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl UmodeUnknownFlag {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for UmodeUnknownFlag {
    fn code(&self) -> u16 {
        codes::ERR_UMODEUNKNOWNFLAG
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<UmodeUnknownFlag> for Message {
    fn from(value: UmodeUnknownFlag) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<UmodeUnknownFlag> for RawMessage {
    fn from(value: UmodeUnknownFlag) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for UmodeUnknownFlag {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<UmodeUnknownFlag, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(UmodeUnknownFlag { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsersDontMatch {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl UsersDontMatch {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for UsersDontMatch {
    fn code(&self) -> u16 {
        codes::ERR_USERSDONTMATCH
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<UsersDontMatch> for Message {
    fn from(value: UsersDontMatch) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<UsersDontMatch> for RawMessage {
    fn from(value: UsersDontMatch) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for UsersDontMatch {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<UsersDontMatch, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(UsersDontMatch { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HelpNotFound {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl HelpNotFound {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn subject(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for HelpNotFound {
    fn code(&self) -> u16 {
        codes::ERR_HELPNOTFOUND
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<HelpNotFound> for Message {
    fn from(value: HelpNotFound) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<HelpNotFound> for RawMessage {
    fn from(value: HelpNotFound) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for HelpNotFound {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<HelpNotFound, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(HelpNotFound { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidKey {
    parameters: ParameterList,
    client: ReplyTarget,
    channel: Channel,
}

impl InvalidKey {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for InvalidKey {
    fn code(&self) -> u16 {
        codes::ERR_INVALIDKEY
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<InvalidKey> for Message {
    fn from(value: InvalidKey) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<InvalidKey> for RawMessage {
    fn from(value: InvalidKey) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for InvalidKey {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<InvalidKey, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let channel = Channel::try_from(String::from(p))?;
        Ok(InvalidKey {
            parameters,
            client,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartTLS {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl StartTLS {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for StartTLS {
    fn code(&self) -> u16 {
        codes::RPL_STARTTLS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<StartTLS> for Message {
    fn from(value: StartTLS) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<StartTLS> for RawMessage {
    fn from(value: StartTLS) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for StartTLS {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<StartTLS, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(StartTLS { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoIsSecure {
    parameters: ParameterList,
    client: ReplyTarget,
    nickname: Nickname,
}

impl WhoIsSecure {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for WhoIsSecure {
    fn code(&self) -> u16 {
        codes::RPL_WHOISSECURE
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<WhoIsSecure> for Message {
    fn from(value: WhoIsSecure) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<WhoIsSecure> for RawMessage {
    fn from(value: WhoIsSecure) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for WhoIsSecure {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<WhoIsSecure, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let nickname = Nickname::try_from(String::from(p))?;
        Ok(WhoIsSecure {
            parameters,
            client,
            nickname,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartTLSError {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl StartTLSError {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for StartTLSError {
    fn code(&self) -> u16 {
        codes::ERR_STARTTLSERROR
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<StartTLSError> for Message {
    fn from(value: StartTLSError) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<StartTLSError> for RawMessage {
    fn from(value: StartTLSError) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for StartTLSError {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<StartTLSError, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(StartTLSError { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidModeParam {
    parameters: ParameterList,
    client: ReplyTarget,
    target: ModeTarget,
}

impl InvalidModeParam {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn target(&self) -> &ModeTarget {
        &self.target
    }

    pub fn modechar(&self) -> &str {
        let Some(p) = self.parameters.get(2) else {
            unreachable!("index 2 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn parameter(&self) -> &str {
        let Some(p) = self.parameters.get(3) else {
            unreachable!("index 3 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for InvalidModeParam {
    fn code(&self) -> u16 {
        codes::ERR_INVALIDMODEPARAM
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<InvalidModeParam> for Message {
    fn from(value: InvalidModeParam) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<InvalidModeParam> for RawMessage {
    fn from(value: InvalidModeParam) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for InvalidModeParam {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<InvalidModeParam, ReplyError> {
        if parameters.len() < 5 {
            return Err(ReplyError::ParamQty {
                min_required: 5,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 5");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 5");
        let target = ModeTarget::try_from(String::from(p))?;
        Ok(InvalidModeParam {
            parameters,
            client,
            target,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HelpStart {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl HelpStart {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn subject(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for HelpStart {
    fn code(&self) -> u16 {
        codes::RPL_HELPSTART
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<HelpStart> for Message {
    fn from(value: HelpStart) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<HelpStart> for RawMessage {
    fn from(value: HelpStart) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for HelpStart {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<HelpStart, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(HelpStart { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HelpTxt {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl HelpTxt {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn subject(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for HelpTxt {
    fn code(&self) -> u16 {
        codes::RPL_HELPTXT
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<HelpTxt> for Message {
    fn from(value: HelpTxt) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<HelpTxt> for RawMessage {
    fn from(value: HelpTxt) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for HelpTxt {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<HelpTxt, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(HelpTxt { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndOfHelp {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl EndOfHelp {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn subject(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for EndOfHelp {
    fn code(&self) -> u16 {
        codes::RPL_ENDOFHELP
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<EndOfHelp> for Message {
    fn from(value: EndOfHelp) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<EndOfHelp> for RawMessage {
    fn from(value: EndOfHelp) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for EndOfHelp {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<EndOfHelp, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(EndOfHelp { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoPrivs {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl NoPrivs {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn privilege(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NoPrivs {
    fn code(&self) -> u16 {
        codes::ERR_NOPRIVS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NoPrivs> for Message {
    fn from(value: NoPrivs) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NoPrivs> for RawMessage {
    fn from(value: NoPrivs) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NoPrivs {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NoPrivs, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(NoPrivs { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoggedIn {
    parameters: ParameterList,
    client: ReplyTarget,
    your_source: ClientSource,
}

impl LoggedIn {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn your_source(&self) -> &ClientSource {
        &self.your_source
    }

    pub fn account(&self) -> &str {
        let Some(p) = self.parameters.get(2) else {
            unreachable!("index 2 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for LoggedIn {
    fn code(&self) -> u16 {
        codes::RPL_LOGGEDIN
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<LoggedIn> for Message {
    fn from(value: LoggedIn) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<LoggedIn> for RawMessage {
    fn from(value: LoggedIn) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for LoggedIn {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<LoggedIn, ReplyError> {
        if parameters.len() < 4 {
            return Err(ReplyError::ParamQty {
                min_required: 4,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 4");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 4");
        let your_source = ClientSource::try_from(String::from(p))?;
        Ok(LoggedIn {
            parameters,
            client,
            your_source,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoggedOut {
    parameters: ParameterList,
    client: ReplyTarget,
    your_source: ClientSource,
}

impl LoggedOut {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn your_source(&self) -> &ClientSource {
        &self.your_source
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for LoggedOut {
    fn code(&self) -> u16 {
        codes::RPL_LOGGEDOUT
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<LoggedOut> for Message {
    fn from(value: LoggedOut) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<LoggedOut> for RawMessage {
    fn from(value: LoggedOut) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for LoggedOut {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<LoggedOut, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        let p = parameters
            .get(1)
            .expect("Parameter 1 should exist when list length is at least 3");
        let your_source = ClientSource::try_from(String::from(p))?;
        Ok(LoggedOut {
            parameters,
            client,
            your_source,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NickLocked {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl NickLocked {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for NickLocked {
    fn code(&self) -> u16 {
        codes::ERR_NICKLOCKED
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<NickLocked> for Message {
    fn from(value: NickLocked) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<NickLocked> for RawMessage {
    fn from(value: NickLocked) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for NickLocked {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<NickLocked, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(NickLocked { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaslSuccess {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl SaslSuccess {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for SaslSuccess {
    fn code(&self) -> u16 {
        codes::RPL_SASLSUCCESS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<SaslSuccess> for Message {
    fn from(value: SaslSuccess) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<SaslSuccess> for RawMessage {
    fn from(value: SaslSuccess) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for SaslSuccess {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<SaslSuccess, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(SaslSuccess { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaslFail {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl SaslFail {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for SaslFail {
    fn code(&self) -> u16 {
        codes::ERR_SASLFAIL
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<SaslFail> for Message {
    fn from(value: SaslFail) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<SaslFail> for RawMessage {
    fn from(value: SaslFail) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for SaslFail {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<SaslFail, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(SaslFail { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaslTooLong {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl SaslTooLong {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for SaslTooLong {
    fn code(&self) -> u16 {
        codes::ERR_SASLTOOLONG
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<SaslTooLong> for Message {
    fn from(value: SaslTooLong) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<SaslTooLong> for RawMessage {
    fn from(value: SaslTooLong) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for SaslTooLong {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<SaslTooLong, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(SaslTooLong { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaslAborted {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl SaslAborted {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for SaslAborted {
    fn code(&self) -> u16 {
        codes::ERR_SASLABORTED
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<SaslAborted> for Message {
    fn from(value: SaslAborted) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<SaslAborted> for RawMessage {
    fn from(value: SaslAborted) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for SaslAborted {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<SaslAborted, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(SaslAborted { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaslAlready {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl SaslAlready {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for SaslAlready {
    fn code(&self) -> u16 {
        codes::ERR_SASLALREADY
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        true
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<SaslAlready> for Message {
    fn from(value: SaslAlready) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<SaslAlready> for RawMessage {
    fn from(value: SaslAlready) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for SaslAlready {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<SaslAlready, ReplyError> {
        if parameters.len() < 2 {
            return Err(ReplyError::ParamQty {
                min_required: 2,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 2");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(SaslAlready { parameters, client })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaslMechs {
    parameters: ParameterList,
    client: ReplyTarget,
}

impl SaslMechs {
    pub fn client(&self) -> &ReplyTarget {
        &self.client
    }

    pub fn mechanisms(&self) -> &str {
        let Some(p) = self.parameters.get(1) else {
            unreachable!("index 1 should exist in reply parameters");
        };
        p.as_str()
    }

    pub fn message(&self) -> &str {
        let Some(p) = self.parameters.last() else {
            unreachable!("reply parameters should be nonempty");
        };
        p.as_str()
    }
}

impl ReplyParts for SaslMechs {
    fn code(&self) -> u16 {
        codes::RPL_SASLMECHS
    }

    fn parameters(&self) -> &ParameterList {
        &self.parameters
    }

    fn is_error(&self) -> bool {
        false
    }

    fn into_parts(self) -> (u16, ParameterList) {
        let code = self.code();
        (code, self.parameters)
    }
}

impl From<SaslMechs> for Message {
    fn from(value: SaslMechs) -> Message {
        Message::from(Reply::from(value))
    }
}

impl From<SaslMechs> for RawMessage {
    fn from(value: SaslMechs) -> RawMessage {
        RawMessage::from(Reply::from(value))
    }
}

impl TryFrom<ParameterList> for SaslMechs {
    type Error = ReplyError;

    fn try_from(parameters: ParameterList) -> Result<SaslMechs, ReplyError> {
        if parameters.len() < 3 {
            return Err(ReplyError::ParamQty {
                min_required: 3,
                received: parameters.len(),
            });
        }
        let p = parameters
            .get(0)
            .expect("Parameter 0 should exist when list length is at least 3");
        let client = ReplyTarget::try_from(String::from(p))?;
        Ok(SaslMechs { parameters, client })
    }
}
