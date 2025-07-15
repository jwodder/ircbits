pub type ParseVerbError = strum::ParseError;

#[derive(strum::AsRefStr, Clone, Debug, strum::Display, strum::EnumString, Eq, Hash, PartialEq)]
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

impl Verb {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl From<String> for Verb {
    fn from(s: String) -> Verb {
        s.parse().expect("Parsing to Verb should always succeed")
    }
}

impl PartialEq<String> for Verb {
    fn eq(&self, other: &String) -> bool {
        self.as_ref() == other.as_str()
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
