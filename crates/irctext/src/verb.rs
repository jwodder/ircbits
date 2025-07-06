#[derive(strum::AsRefStr, Clone, Debug, strum::Display, strum::EnumString, Eq, PartialEq)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Verb {
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
    #[strum(default, transparent)]
    Unknown(String),
}

impl TryFrom<String> for Verb {
    type Error = strum::ParseError;

    fn try_from(s: String) -> Result<Verb, strum::ParseError> {
        s.parse()
    }
}

impl PartialEq<str> for Verb {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl<'a> PartialEq<&'a str> for Verb {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}
