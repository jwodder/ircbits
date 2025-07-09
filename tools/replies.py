#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum
import re
from typing import Literal


class IrcType(Enum):
    Str = "&str"
    U16 = "u16"
    U32 = "u32"
    U64 = "u64"
    String = "String"
    Channel = "Channel"
    ChannelStatus = "ChannelStatus"
    ClientSource = "ClientSource"
    ISupportParam = "ISupportParam"
    ModeString = "ModeString"
    ModeTarget = "ModeTarget"
    MsgTarget = "MsgTarget"
    Nickname = "Nickname"
    OptHost = "Option<Host>"
    OptIpAddr = "Option<IpAddr>"
    OptUsername = "Option<Username>"
    ParameterList = "ParameterList"
    ReplyTarget = "ReplyTarget"
    Username = "Username"
    VecPrefixChannel = "Vec<(Option<char>, Channel)>"
    VecPrefixNickname = "Vec<(Option<char>, Nickname)>"
    VecUserHostReply = "Vec<UserHostReply>"
    Verb = "Verb"
    WhoFlags = "WhoFlags"

    def field_type(self, param: Param) -> str:
        if param.optional:
            return f"Option<{self.value}>"
        elif param.upto is not None:
            return f"Vec<{self.value}>"
        else:
            return self.value

    def getter(self, fieldname: str, param: Param) -> str:
        optional = param.optional
        index = param.index
        if self is IrcType.Str:
            if param.word is not None:
                assert not optional and index is Last and param.word == "remainder"
                return (
                    f"    pub fn {fieldname}(&self) -> &str {{\n"
                    "        let Some(p) = self.parameters.last() else {\n"
                    '            unreachable!("reply parameters should be nonempty");\n'
                    "        };\n"
                    "        split_word(p.as_str()).1\n"
                    "    }"
                )
            elif optional:
                assert isinstance(
                    index, Maybe
                ), f"index {index!r} for type {self} is not a Maybe"
                return (
                    f"    pub fn {fieldname}(&self) -> Option<&str> {{\n"
                    f"        self.parameters.get({index.index}).map(|p| p.as_str())\n"
                    "    }"
                )
            elif index is LastType.Last:
                return (
                    f"    pub fn {fieldname}(&self) -> &str {{\n"
                    "        let Some(p) = self.parameters.last() else {\n"
                    '            unreachable!("reply parameters should be nonempty");\n'
                    "        };\n"
                    "        p.as_str()\n"
                    "    }"
                )
            else:
                assert isinstance(
                    index, int
                ), f"index {index!r} for type {self} is not an int"
                return (
                    f"    pub fn {fieldname}(&self) -> &str {{\n"
                    f"        let Some(p) = self.parameters.get({index}) else {{\n"
                    f'            unreachable!("index {index} should exist in reply parameters");\n'
                    "        };\n"
                    "        p.as_str()\n"
                    "    }"
                )
        else:
            match self:
                case IrcType.U16 | IrcType.U32 | IrcType.U64:
                    if optional:
                        rettype = f"Option<{self.value}>"
                    else:
                        rettype = self.value
                    getting = f"self.{fieldname}"
                case IrcType.OptHost | IrcType.OptUsername:
                    rettype = re.sub(r"^(Option<)(.+>)$", r"\1&\2", self.value)
                    getting = f"self.{fieldname}.as_ref()"
                case IrcType.OptIpAddr:
                    rettype = self.value
                    getting = f"self.{fieldname}"
                case (
                    IrcType.VecPrefixChannel
                    | IrcType.VecPrefixNickname
                    | IrcType.VecUserHostReply
                ):
                    assert not optional
                    rettype = (
                        "&[" + self.value.removeprefix("Vec<").removesuffix(">") + "]"
                    )
                    getting = f"&self.{fieldname}"
                case _:
                    if optional:
                        rettype = f"Option<&{self.value}>"
                        getting = f"self.{fieldname}.as_ref()"
                    elif param.upto is not None:
                        rettype = f"&[{self.value}]"
                        getting = f"&self.{fieldname}"
                    else:
                        rettype = f"&{self.value}"
                        getting = f"&self.{fieldname}"
            return "".join(
                [
                    f"    pub fn {fieldname}(&self) -> {rettype} {{\n",
                    f"        {getting}\n",
                    "    }",
                ]
            )

    def parse(self, argname: str, in_closure: bool, is_str: bool = False) -> str:
        match self:
            case (
                IrcType.Channel
                | IrcType.ChannelStatus
                | IrcType.ClientSource
                | IrcType.ISupportParam
                | IrcType.ModeString
                | IrcType.ModeTarget
                | IrcType.Nickname
                | IrcType.ReplyTarget
                | IrcType.MsgTarget
                | IrcType.Username
                | IrcType.WhoFlags
            ):
                if in_closure:
                    return f"{self.value}::try_from(String::from({argname}))"
                else:
                    return f"{self.value}::try_from(String::from({argname}))?"
            case IrcType.Verb:
                return f"{self.value}::from(String::from({argname}))"
            case IrcType.U16 | IrcType.U32 | IrcType.U64:
                this = argname if is_str else f"{argname}.as_str()"
                if in_closure:
                    # Let `cargo fmt` take care of the indentation here
                    return (
                        f"match {this}.parse::<{self.value}>() {{\n"
                        "    Ok(n) => Ok(n),\n"
                        "    Err(inner) => {\n"
                        "        Err(ReplyError::Int {\n"
                        f"            string: String::from({argname}),\n"
                        "            inner,\n"
                        "        })\n"
                        "    }\n"
                        "}"
                    )
                else:
                    return (
                        f"match {this}.parse::<{self.value}>() {{\n"
                        "            Ok(n) => n,\n"
                        "            Err(inner) => {\n"
                        "                return Err(ReplyError::Int {\n"
                        f"                    string: String::from({argname}),\n"
                        "                    inner,\n"
                        "                })\n"
                        "            }\n"
                        "        }"
                    )
            case IrcType.VecPrefixChannel:
                assert not in_closure
                return (
                    f"split_spaces({argname}.as_str()).map(|s| {{\n"
                    "            let (prefix, s) = pop_channel_membership(s);\n"
                    "            Channel::try_from(s.to_owned()).map(|chan| (prefix, chan))\n"
                    "        }).collect::<Result<Vec<_>, _>>()?"
                )
            case IrcType.VecPrefixNickname:
                assert not in_closure
                return (
                    f"split_spaces({argname}.as_str()).map(|s| {{\n"
                    "            let (prefix, s) = pop_channel_membership(s);\n"
                    "            Nickname::try_from(s.to_owned()).map(|nick| (prefix, nick))\n"
                    "        }).collect::<Result<Vec<_>, _>>()?"
                )
            case IrcType.VecUserHostReply:
                assert not in_closure
                return f"split_spaces({argname}.as_str()).map(|s| UserHostReply::try_from(s.to_owned())).collect::<Result<Vec<_>, _>>()?"
            case (
                IrcType.Str
                | IrcType.String
                | IrcType.OptHost
                | IrcType.OptIpAddr
                | IrcType.OptUsername
                | IrcType.ParameterList
            ):
                raise AssertionError(
                    "IrcType.parse() called on unsupported receiver: {self!r}"
                )
            case _:
                raise AssertionError("Unhandled IrcType.parse() case: {self!r}")


class LastType(Enum):
    Last = 1
    Remainder = 2


Last = LastType.Last
Remainder = LastType.Remainder


@dataclass
class Maybe:
    index: int


type Index = int | Maybe | LastType


@dataclass
class Param:
    index: Index | Literal["todo"]
    type: IrcType = IrcType.Str
    word: int | Literal["remainder"] | None = None
    upto: Index | None = None

    @property
    def optional(self) -> bool:
        return isinstance(self.index, Maybe)


@dataclass
class Reply:
    const: str
    name: str
    code: int
    minlength: int
    params: dict[str, Param] = field(default_factory=dict)
    error: bool = False

    @property
    def uses_last(self) -> bool:
        return any(p.index is Last for p in self.params.values())


REPLIES = [
    Reply(
        const="RPL_WELCOME",
        name="Welcome",
        code=1,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_YOURHOST",
        name="YourHost",
        code=2,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_CREATED",
        name="Created",
        code=3,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_MYINFO",
        name="MyInfo",
        code=4,
        minlength=5,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "servername": Param(1),
            "version": Param(2),
            "available_user_modes": Param(3),
            "available_channel_modes": Param(4),
            "channel_modes_with_param": Param(Maybe(5)),
        },
    ),
    Reply(
        const="RPL_ISUPPORT",
        name="ISupport",
        code=5,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "tokens": Param(index=1, upto=Last, type=IrcType.ISupportParam),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_REMOTEISUPPORT",
        name="RemoteISupport",
        code=105,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "tokens": Param(index=1, upto=Last, type=IrcType.ISupportParam),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_BOUNCE",
        name="Bounce",
        code=10,
        minlength=4,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "hostname": Param(1),
            "port": Param(index=2, type=IrcType.U16),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_STATSCOMMANDS",
        name="StatsCommands",
        code=212,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "command": Param(1),
            "count": Param(index=2, type=IrcType.U64),
            "byte_count": Param(index=Maybe(3), type=IrcType.U64),
            "remote_count": Param(index=Maybe(4), type=IrcType.U64),
        },
    ),
    Reply(
        const="RPL_ENDOFSTATS",
        name="EndOfStats",
        code=219,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "stats_letter": Param(1),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_UMODEIS",
        name="UModeIs",
        code=221,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "user_modes": Param(1),
        },
    ),
    Reply(
        const="RPL_STATSUPTIME",
        name="StatsUptime",
        code=242,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_LUSERCLIENT",
        name="LuserClient",
        code=251,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_LUSEROP",
        name="LuserOp",
        code=252,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "ops": Param(index=1, type=IrcType.U64),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_LUSERUNKNOWN",
        name="LuserUnknown",
        code=253,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "connections": Param(index=1, type=IrcType.U64),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_LUSERCHANNELS",
        name="LuserChannels",
        code=254,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channels": Param(index=1, type=IrcType.U64),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_LUSERME",
        name="LuserMe",
        code=255,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_ADMINME",
        name="AdminMe",
        code=256,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "server": Param(Maybe(1)),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_ADMINLOC1",
        name="AdminLoc1",
        code=257,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_ADMINLOC2",
        name="AdminLoc2",
        code=258,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_ADMINEMAIL",
        name="AdminEmail",
        code=259,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_TRYAGAIN",
        name="TryAgain",
        code=263,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "command": Param(index=1, type=IrcType.Verb),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_LOCALUSERS",
        name="LocalUsers",
        code=265,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "current_users": Param(index=Maybe(1), type=IrcType.U64),
            "max_users": Param(index=Maybe(2), type=IrcType.U64),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_GLOBALUSERS",
        name="GlobalUsers",
        code=266,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "current_users": Param(index=Maybe(1), type=IrcType.U64),
            "max_users": Param(index=Maybe(2), type=IrcType.U64),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_WHOISCERTFP",
        name="WhoIsCertFP",
        code=276,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "message": Param(Last),
        },
    ),
    Reply(const="RPL_NONE", name="None", code=300, minlength=0, params={}),
    Reply(
        const="RPL_AWAY",
        name="Away",
        code=301,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_USERHOST",
        name="UserHostRpl",
        code=302,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "replies": Param(index=Last, type=IrcType.VecUserHostReply),
        },
    ),
    Reply(
        const="RPL_UNAWAY",
        name="UnAway",
        code=305,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_NOWAWAY",
        name="NowAway",
        code=306,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_WHOISREGNICK",
        name="WhoIsRegNick",
        code=307,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_WHOISUSER",
        name="WhoIsUser",
        code=311,
        minlength=6,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "username": Param(index=2, type=IrcType.Username),
            "host": Param(3),
            "realname": Param(5),
        },
    ),
    Reply(
        const="RPL_WHOISSERVER",
        name="WhoIsServer",
        code=312,
        minlength=4,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "server": Param(2),
            "server_info": Param(Last),
        },
    ),
    Reply(
        const="RPL_WHOISOPERATOR",
        name="WhoIsOperator",
        code=313,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_WHOWASUSER",
        name="WhoWasUser",
        code=314,
        minlength=6,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "username": Param(index=2, type=IrcType.Username),
            "host": Param(3),
            "realname": Param(5),
        },
    ),
    Reply(
        const="RPL_ENDOFWHO",
        name="EndOfWho",
        code=315,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "mask": Param(1),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_WHOISIDLE",
        name="WhoIsIdle",
        code=317,
        minlength=5,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "secs": Param(index=2, type=IrcType.U64),
            "signon": Param(index=3, type=IrcType.U64),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_ENDOFWHOIS",
        name="EndOfWhoIs",
        code=318,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_WHOISCHANNELS",
        name="WhoIsChannels",
        code=319,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "channels": Param(index=Last, type=IrcType.VecPrefixChannel),
        },
    ),
    Reply(
        const="RPL_WHOISSPECIAL",
        name="WhoIsSpecial",
        code=320,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_LISTSTART",
        name="ListStart",
        code=321,
        minlength=3,
        params={"client": Param(0)},
    ),
    Reply(
        const="RPL_LIST",
        name="List",
        code=322,
        minlength=4,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "clients": Param(index=2, type=IrcType.U64),
            "topic": Param(Last),
        },
    ),
    Reply(
        const="RPL_LISTEND",
        name="ListEnd",
        code=323,
        minlength=2,
        params={"client": Param(0)},
    ),
    Reply(
        const="RPL_CHANNELMODEIS",
        name="ChannelModeIs",
        code=324,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "modestring": Param(index=2, type=IrcType.ModeString),
            "arguments": Param(index=Remainder, type=IrcType.ParameterList),
        },
    ),
    Reply(
        const="RPL_CREATIONTIME",
        name="CreationTime",
        code=329,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "creationtime": Param(index=2, type=IrcType.U64),
        },
    ),
    Reply(
        const="RPL_WHOISACCOUNT",
        name="WhoIsAccount",
        code=330,
        minlength=4,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "account": Param(2),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_NOTOPIC",
        name="NoTopic",
        code=331,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_TOPIC",
        name="Topic",
        code=332,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "topic": Param(Last),
        },
    ),
    Reply(
        const="RPL_TOPICWHOTIME",
        name="TopicWhoTime",
        code=333,
        minlength=4,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "nickname": Param(index=2, type=IrcType.Nickname),
            "setat": Param(index=3, type=IrcType.U64),
        },
    ),
    Reply(
        const="RPL_INVITELIST",
        name="InviteList",
        code=336,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
        },
    ),
    Reply(
        const="RPL_ENDOFINVITELIST",
        name="EndOfInviteList",
        code=337,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_WHOISACTUALLY",
        name="WhoIsActually",
        code=338,
        minlength=3,
        params={
            # "<client> <nickname> :is actually ..."
            # "<client> <nickname> <host|ip> :Is actually using host"
            # "<client> <nickname> <username>@<hostname> <ip> :Is actually using host"
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "host": Param(index="todo", type=IrcType.OptHost),
            "username": Param(index="todo", type=IrcType.OptUsername),
            "ip": Param(index="todo", type=IrcType.OptIpAddr),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_INVITING",
        name="Inviting",
        code=341,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "channel": Param(index=2, type=IrcType.Channel),
        },
    ),
    Reply(
        const="RPL_INVEXLIST",
        name="InvExList",
        code=346,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "mask": Param(2),
        },
    ),
    Reply(
        const="RPL_ENDOFINVEXLIST",
        name="EndOfInvExList",
        code=347,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_EXCEPTLIST",
        name="ExceptList",
        code=348,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "mask": Param(2),
        },
    ),
    Reply(
        const="RPL_ENDOFEXCEPTLIST",
        name="EndOfExceptList",
        code=349,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_VERSION",
        name="Version",
        code=351,
        minlength=4,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "version": Param(1),
            "server": Param(2),
            "comments": Param(Last),
        },
    ),
    Reply(
        const="RPL_WHOREPLY",
        name="WhoReply",
        code=352,
        minlength=8,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "username": Param(index=2, type=IrcType.Username),
            "host": Param(3),
            "server": Param(4),
            "nickname": Param(index=5, type=IrcType.Nickname),
            "flags": Param(index=6, type=IrcType.WhoFlags),
            "hopcount": Param(index=Last, type=IrcType.U32, word=0),
            "realname": Param(index=Last, word="remainder"),
        },
    ),
    Reply(
        const="RPL_NAMREPLY",
        name="NamReply",
        code=353,
        minlength=4,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel_status": Param(index=1, type=IrcType.ChannelStatus),
            "channel": Param(index=2, type=IrcType.Channel),
            "clients": Param(index=Last, type=IrcType.VecPrefixNickname),
        },
    ),
    Reply(
        const="RPL_LINKS",
        name="Links",
        code=364,
        minlength=4,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "server1": Param(1),
            "server2": Param(2),
            "hopcount": Param(index=Last, type=IrcType.U32, word=0),
            "server_info": Param(index=Last, word="remainder"),
        },
    ),
    Reply(
        const="RPL_ENDOFLINKS",
        name="EndOfLinks",
        code=365,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_ENDOFNAMES",
        name="EndOfNames",
        code=366,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_BANLIST",
        name="BanList",
        code=367,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "mask": Param(2),
            "who": Param(Maybe(3)),
            "set_ts": Param(index=Maybe(4), type=IrcType.U64),
        },
    ),
    Reply(
        const="RPL_ENDOFBANLIST",
        name="EndOfBanList",
        code=368,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_ENDOFWHOWAS",
        name="EndOfWhoWas",
        code=369,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_INFO",
        name="Info",
        code=371,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_MOTD",
        name="Motd",
        code=372,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_ENDOFINFO",
        name="EndOfInfo",
        code=374,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_MOTDSTART",
        name="MotdStart",
        code=375,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_ENDOFMOTD",
        name="EndOfMotd",
        code=376,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_WHOISHOST",
        name="WhoIsHost",
        code=378,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_WHOISMODES",
        name="WhoIsModes",
        code=379,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_YOUREOPER",
        name="YoureOper",
        code=381,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_REHASHING",
        name="Rehashing",
        code=382,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "config_file": Param(1),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_TIME",
        name="Time",
        code=391,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "server": Param(1),
            "timestamp": Param(index=Maybe(2), type=IrcType.U64),
            "ts_offset": Param(Maybe(3)),
            "human_time": Param(Last),
        },
    ),
    Reply(
        const="ERR_UNKNOWNERROR",
        name="UnknownError",
        code=400,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "command": Param(index=1, type=IrcType.Verb),
            "subcommands": Param(index=2, upto=Last, type=IrcType.String),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NOSUCHNICK",
        name="NoSuchNick",
        code=401,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "target": Param(index=1, type=IrcType.MsgTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NOSUCHSERVER",
        name="NoSuchServer",
        code=402,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "server": Param(1),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NOSUCHCHANNEL",
        name="NoSuchChannel",
        code=403,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_CANNOTSENDTOCHAN",
        name="CannotSendToChan",
        code=404,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_TOOMANYCHANNELS",
        name="TooManyChannels",
        code=405,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_WASNOSUCHNICK",
        name="WasNoSuchNick",
        code=406,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NOORIGIN",
        name="NoOrigin",
        code=409,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NORECIPIENT",
        name="NoRecipient",
        code=411,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NOTEXTTOSEND",
        name="NoTextToSend",
        code=412,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_INPUTTOOLONG",
        name="InputTooLong",
        code=417,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_UNKNOWNCOMMAND",
        name="UnknownCommand",
        code=421,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "command": Param(index=1, type=IrcType.Verb),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NOMOTD",
        name="NoMotd",
        code=422,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NONICKNAMEGIVEN",
        name="NoNicknameGiven",
        code=431,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_ERRONEUSNICKNAME",
        name="ErroneousNickname",
        code=432,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(1),  # NOT a Nickname!
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NICKNAMEINUSE",
        name="NicknameInUse",
        code=433,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NICKCOLLISION",
        name="NickCollision",
        code=436,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_USERNOTINCHANNEL",
        name="UserNotInChannel",
        code=441,
        error=True,
        minlength=4,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "channel": Param(index=2, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NOTONCHANNEL",
        name="NotOnChannel",
        code=442,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_USERONCHANNEL",
        name="UserOnChannel",
        code=443,
        error=True,
        minlength=4,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "channel": Param(index=2, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NOTREGISTERED",
        name="NotRegistered",
        code=451,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NEEDMOREPARAMS",
        name="NeedMoreParams",
        code=461,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "command": Param(index=1, type=IrcType.Verb),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_ALREADYREGISTERED",
        name="AlreadyRegistered",
        code=462,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_PASSWDMISMATCH",
        name="PasswdMismatch",
        code=464,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_YOUREBANNEDCREEP",
        name="YoureBannedCreep",
        code=465,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_CHANNELISFULL",
        name="ChannelIsFull",
        code=471,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_UNKNOWNMODE",
        name="UnknownMode",
        code=472,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "modechar": Param(1),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_INVITEONLYCHAN",
        name="InviteOnlyChan",
        code=473,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_BANNEDFROMCHAN",
        name="BannedFromChan",
        code=474,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_BADCHANNELKEY",
        name="BadChannelKey",
        code=475,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_BADCHANMASK",
        name="BadChanMask",
        code=476,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(1),  # NOT a valid Channel!
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NOPRIVILEGES",
        name="NoPrivileges",
        code=481,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_CHANOPRIVSNEEDED",
        name="ChanOPrivsNeeded",
        code=482,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_CANTKILLSERVER",
        name="CantKillServer",
        code=483,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NOOPERHOST",
        name="NoOperHost",
        code=491,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_UMODEUNKNOWNFLAG",
        name="UmodeUnknownFlag",
        code=501,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_USERSDONTMATCH",
        name="UsersDontMatch",
        code=502,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_HELPNOTFOUND",
        name="HelpNotFound",
        code=524,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "subject": Param(1),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_INVALIDKEY",
        name="InvalidKey",
        code=525,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "channel": Param(index=1, type=IrcType.Channel),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_STARTTLS",
        name="StartTLS",
        code=670,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_WHOISSECURE",
        name="WhoIsSecure",
        code=671,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "nickname": Param(index=1, type=IrcType.Nickname),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_STARTTLSERROR",
        name="StartTLSError",
        code=691,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_INVALIDMODEPARAM",
        name="InvalidModeParam",
        code=696,
        error=True,
        minlength=5,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "target": Param(index=1, type=IrcType.ModeTarget),
            "modechar": Param(2),
            "parameter": Param(3),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_HELPSTART",
        name="HelpStart",
        code=704,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "subject": Param(1),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_HELPTXT",
        name="HelpTxt",
        code=705,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "subject": Param(1),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_ENDOFHELP",
        name="EndOfHelp",
        code=706,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "subject": Param(1),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NOPRIVS",
        name="NoPrivs",
        code=723,
        error=True,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "privilege": Param(1),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_LOGGEDIN",
        name="LoggedIn",
        code=900,
        minlength=4,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "your_source": Param(index=1, type=IrcType.ClientSource),
            "account": Param(2),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_LOGGEDOUT",
        name="LoggedOut",
        code=901,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "your_source": Param(index=1, type=IrcType.ClientSource),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_NICKLOCKED",
        name="NickLocked",
        code=902,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_SASLSUCCESS",
        name="SaslSuccess",
        code=903,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_SASLFAIL",
        name="SaslFail",
        code=904,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_SASLTOOLONG",
        name="SaslTooLong",
        code=905,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_SASLABORTED",
        name="SaslAborted",
        code=906,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="ERR_SASLALREADY",
        name="SaslAlready",
        code=907,
        error=True,
        minlength=2,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "message": Param(Last),
        },
    ),
    Reply(
        const="RPL_SASLMECHS",
        name="SaslMechs",
        code=908,
        minlength=3,
        params={
            "client": Param(index=0, type=IrcType.ReplyTarget),
            "mechanisms": Param(1),
            "message": Param(Last),
        },
    ),
]


def main() -> None:
    print("#![expect(unreachable_code, unused_variables)]")
    print("use crate::types::{")
    print(
        "    Channel, ChannelStatus, ISupportParam, ModeString, ModeTarget, MsgTarget, Nickname,"
    )
    print(
        "    ParseChannelError, ParseChannelStatusError, ParseISupportParamError, ParseModeStringError,"
    )
    print(
        "    ParseModeTargetError, ParseMsgTargetError, ParseNicknameError, ParseReplyTargetError,"
    )
    print(
        "    ParseUserHostReplyError, ParseUsernameError, ParseWhoFlagsError, ReplyTarget, UserHostReply,"
    )
    print("    Username, WhoFlags,")
    print("};")
    print("use crate::util::{pop_channel_membership, split_spaces, split_word};")
    print("use crate::{")
    print(
        "    ClientSource, Message, ParameterList, ParseClientSourceError, ParseVerbError, Payload,"
    )
    print("    RawMessage, TryFromStringError, Verb,")
    print("};")
    print("use enum_dispatch::enum_dispatch;")
    print("use std::net::IpAddr;")
    print("use thiserror::Error;")
    print("use url::Host;")
    print()
    print("#[enum_dispatch]")
    print("pub trait ReplyParts {")
    print("    fn code(&self) -> u16;")
    print("    fn parameters(&self) -> &ParameterList;")
    print("    fn is_error(&self) -> bool;")
    print("    fn into_parts(self) -> (u16, ParameterList);")
    print("}")
    print()
    print("#[enum_dispatch(ReplyParts)] // This also gives us From and TryInto")
    print("#[derive(Clone, Debug, Eq, PartialEq)]")
    print("pub enum Reply {")
    for r in REPLIES:
        print(f"    {r.name},")
    print("}")
    print()

    print("impl Reply {")
    print(
        "    pub fn from_parts(code: u16, params: ParameterList) -> Result<Reply, ReplyError> {"
    )
    print("        match code {")
    for r in REPLIES:
        print(" " * 12 + f"{r.code} => {r.name}::try_from(params).map(Into::into),")
    print(" " * 12 + "_ => Err(ReplyError::Unknown(code)),")
    print("        }")
    print("    }")
    print("}")
    print()

    print("impl From<Reply> for Message {")
    print("    fn from(value: Reply) -> Message {")
    print("        Message {")
    print("            source: None,")
    print("            payload: Payload::Reply(value),")
    print("        }")
    print("    }")
    print("}")
    print()

    print("impl From<Reply> for RawMessage {")
    print("    fn from(value: Reply) -> RawMessage {")
    print("        RawMessage::from(Message::from(value))")
    print("    }")
    print("}")
    print()

    print("#[derive(Clone, Debug, Eq, Error, PartialEq)]")
    print("pub enum ReplyError {")
    print('    #[error("unknown/unrecognized reply code {0:03}")]')
    print("    Unknown(u16),")
    print()
    print(
        '    #[error("invalid number of parameters: at least {min_required} required, {received} received")]'
    )
    print("    ParamQty {")
    print("        min_required: usize,")
    print("        received: usize,")
    print("    },")
    print()
    print('    #[error("failed to parse integer string {string:?}: {inner}")]')
    print("    Int {")
    print("        string: String,")
    print("        inner: std::num::ParseIntError,")
    print("    },")
    print()
    print('    #[error("failed to parse channel string")]')
    print("    Channel(#[from] TryFromStringError<ParseChannelError>),")
    print()
    print('    #[error("failed to parse channel status string")]')
    print("    ChannelStatus(#[from] TryFromStringError<ParseChannelStatusError>),")
    print()
    print('    #[error("failed to parse source string")]')
    print("    ClientSource(#[from] TryFromStringError<ParseClientSourceError>),")
    print()
    print('    #[error("failed to parse RPL_ISUPPORT param")]')
    print("    ISupportParam(#[from] TryFromStringError<ParseISupportParamError>),")
    print()
    print('    #[error("failed to parse mode string")]')
    print("    ModeString(#[from] TryFromStringError<ParseModeStringError>),")
    print()
    print('    #[error("failed to parse mode target string")]')
    print("    ModeTarget(#[from] TryFromStringError<ParseModeTargetError>),")
    print()
    print('    #[error("failed to parse target string")]')
    print("    MsgTarget(#[from] TryFromStringError<ParseMsgTargetError>),")
    print()
    print('    #[error("failed to parse nickname string")]')
    print("    Nickname(#[from] TryFromStringError<ParseNicknameError>),")
    print()
    print('    #[error("failed to parse reply target string")]')
    print("    ReplyTarget(#[from] TryFromStringError<ParseReplyTargetError>),")
    print()
    print('    #[error("failed to parse USERHOST reply string")]')
    print("    UserHostReply(#[from] TryFromStringError<ParseUserHostReplyError>),")
    print()
    print('    #[error("failed to parse username string")]')
    print("    Username(#[from] TryFromStringError<ParseUsernameError>),")
    print()
    print('    #[error("failed to parse verb string")]')
    print("    Verb(#[from] TryFromStringError<ParseVerbError>),")
    print()
    print('    #[error("failed to parse RPL_WHOREPLY flags")]')
    print("    WhoFlags(#[from] TryFromStringError<ParseWhoFlagsError>),")
    print("}")
    print()

    print("pub mod codes {")
    for r in REPLIES:
        print(f"    pub const {r.const}: u16 = {r.code};")
    print("}")

    for r in REPLIES:
        print()
        print("#[derive(Clone, Debug, Eq, PartialEq)]")
        print(f"pub struct {r.name} {{")
        print("    parameters: ParameterList,")
        for name, p in r.params.items():
            if p.type is not IrcType.Str:
                print(f"    {name}: " + p.type.field_type(p) + ",")
        print("}")

        if r.params:
            print()
            print(f"impl {r.name} {{")
            first = True
            for name, p in r.params.items():
                if first:
                    first = False
                else:
                    print()
                print(p.type.getter(name, p))
            print("}")

        print()
        print(f"impl ReplyParts for {r.name} {{")
        print("    fn code(&self) -> u16 {")
        print(f"        {r.code}")
        print("    }")
        print()
        print("    fn parameters(&self) -> &ParameterList {")
        print("        &self.parameters")
        print("    }")
        print()
        print("    fn is_error(&self) -> bool {")
        print("        " + ("true" if r.error else "false"))
        print("    }")
        print()
        print("    fn into_parts(self) -> (u16, ParameterList) {")
        print("        let code = self.code();")
        print("        (code, self.parameters)")
        print("    }")
        print("}")

        print()
        print(f"impl From<{r.name}> for Message {{")
        print(f"    fn from(value: {r.name}) -> Message {{")
        print("        Message::from(Reply::from(value))")
        print("    }")
        print("}")

        print()
        print(f"impl From<{r.name}> for RawMessage {{")
        print(f"    fn from(value: {r.name}) -> RawMessage {{")
        print("        RawMessage::from(Reply::from(value))")
        print("    }")
        print("}")

        print()
        print(f"impl TryFrom<ParameterList> for {r.name} {{")
        print("    type Error = ReplyError;")
        print()
        print(
            f"    fn try_from(parameters: ParameterList) -> Result<{r.name}, ReplyError> {{"
        )
        if r.minlength == 0:
            print(f"        Ok({r.name} {{ parameters }})")
        else:
            print(f"        if parameters.len() < {r.minlength} {{")
            print("            return Err(ReplyError::ParamQty {")
            print(f"                min_required: {r.minlength},")
            print("                received: parameters.len(),")
            print("            });")
            print("        }")
            fields = ["parameters"]
            prev_index = None
            for name, p in r.params.items():
                if p.type is IrcType.Str:
                    continue
                if isinstance(p.index, int):
                    assert (
                        p.word is None
                    ), f"Param {name} of {r.name} has word={p.word!r} and non-Last index {p.index}"
                    if p.upto is None:
                        print("        let p = parameters")
                        print(f"            .get({p.index})")
                        print(
                            f'            .expect("Parameter {p.index} should exist when list length is at least {r.minlength}");'
                        )
                        print(f"        let {name} = " + p.type.parse("p", False) + ";")
                    else:
                        assert p.upto is Last
                        if p.type is IrcType.String:
                            print(f"        let {name} = parameters")
                            print("            .iter()")
                            print(f"            .skip({p.index})")
                            print(
                                f"            .take(parameters.len() - {p.index + 1})"
                            )
                            print("            .map(String::from)")
                            print("            .collect::<Vec<_>>();")
                        else:
                            print(f"        let {name} = parameters")
                            print("            .iter()")
                            print(f"            .skip({p.index})")
                            print(
                                f"            .take(parameters.len() - {p.index + 1})"
                            )
                            print(f"            .map(|p| {p.type.parse('p', True)})")
                            print("            .collect::<Result<Vec<_>, _>>()?;")
                else:
                    assert (
                        p.upto is None
                    ), f"Param {name} of {r.name} has upto={p.upto!r} and non-int index {p.index!r}"
                    if isinstance(p.index, Maybe):
                        assert (
                            p.word is None
                        ), f"Param {name} of {r.name} has word={p.word!r} and non-Last index {p.index}"
                        mindex = p.index.index
                        if r.uses_last:
                            # Be sure not to steal Last!
                            print(
                                f"        let {name} = (parameters.len() > {mindex + 1}).then(|| {{"
                            )
                            print("            let p = parameters")
                            print(f"                .get({mindex})")
                            print(
                                f'                .expect("Parameter {mindex} should exist when list length is at least {r.minlength}");'
                            )
                            print(f'            {p.type.parse("p", True)}')
                            print("        }).transpose()?;")
                        else:
                            print(
                                f"        let {name} = parameters.get({mindex}).map(|p| {p.type.parse('p', True)}).transpose()?;"
                            )
                    elif p.index is Remainder:
                        assert (
                            p.word is None
                        ), f"Param {name} of {r.name} has word={p.word!r} and non-Last index {p.index}"
                        assert p.type is IrcType.ParameterList
                        assert isinstance(prev_index, int)
                        print("        let mut iter = parameters.clone().into_iter();")
                        print(f"        for _ in 0..{prev_index} {{")
                        print("            let _ = iter.next();")
                        print("        }")
                        print(f"        let {name} = iter.into_parameter_list();")
                    elif p.index is Last:
                        print(
                            f'        let p = parameters.last().expect("Parameter list should be nonempty when list length is at least {r.minlength}");'
                        )
                        if p.word is None:
                            print(
                                f"        let {name} = "
                                + p.type.parse("p", False)
                                + ";"
                            )
                        else:
                            assert p.word == 0
                            print("        let word = split_word(p.as_str()).0;")
                            print(
                                f"        let {name} = {p.type.parse('word', False, is_str=True)};"
                            )
                    elif p.index == "todo":
                        print(f"        let {name} = todo!();")
                    else:
                        raise AssertionError(
                            f"unhandled index for param {name} of {r.name}: {p.index!r}"
                        )
                fields.append(name)
                prev_index = p.index
            print(f"        Ok({r.name} {{ " + ", ".join(fields) + " })")
        print("    }")
        print("}")


if __name__ == "__main__":
    main()
