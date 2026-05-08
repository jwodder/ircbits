use crate::validstr::TryFromStringError;
use crate::{MiddleParam, TrailingParam};
use std::cmp::Ordering;
use thiserror::Error;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Capability(InnerCap);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum InnerCap {
    Std(StandardCap),
    Custom(String),
}

impl PartialOrd for Capability {
    fn partial_cmp(&self, other: &Capability) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Capability {
    fn cmp(&self, other: &Self) -> Ordering {
        let left = self.as_str();
        let right = other.as_str();
        left.cmp(right)
    }
}

impl Capability {
    pub fn as_str(&self) -> &str {
        match &self.0 {
            InnerCap::Std(c) => c.as_str(),
            InnerCap::Custom(s) => s.as_str(),
        }
    }
}

impl From<Capability> for String {
    fn from(value: Capability) -> String {
        match value.0 {
            InnerCap::Std(c) => c.as_str().to_owned(),
            InnerCap::Custom(s) => s,
        }
    }
}

impl From<&Capability> for String {
    fn from(value: &Capability) -> String {
        match &value.0 {
            InnerCap::Std(c) => c.as_str().to_owned(),
            InnerCap::Custom(s) => s.clone(),
        }
    }
}

impl std::fmt::Debug for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PartialEq<String> for Capability {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<str> for Capability {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<'a> PartialEq<&'a str> for Capability {
    fn eq(&self, other: &&'a str) -> bool {
        &self.as_str() == other
    }
}

impl AsRef<str> for Capability {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::borrow::Borrow<str> for Capability {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl std::ops::Deref for Capability {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl std::str::FromStr for Capability {
    type Err = ParseCapabilityError;

    fn from_str(s: &str) -> Result<Capability, ParseCapabilityError> {
        if let Some(c) = StandardCap::from_str(s) {
            Ok(Capability(InnerCap::Std(c)))
        } else {
            match validate_capability(s) {
                Ok(()) => Ok(Capability(InnerCap::Custom(s.to_owned()))),
                Err(e) => Err(e),
            }
        }
    }
}

impl TryFrom<String> for Capability {
    type Error = TryFromStringError<ParseCapabilityError>;

    fn try_from(string: String) -> Result<Capability, Self::Error> {
        if let Some(c) = StandardCap::from_str(&string) {
            Ok(Capability(InnerCap::Std(c)))
        } else {
            match validate_capability(&string) {
                Ok(()) => Ok(Capability(InnerCap::Custom(string))),
                Err(e) => Err(TryFromStringError { inner: e, string }),
            }
        }
    }
}

impl From<Capability> for MiddleParam {
    fn from(value: Capability) -> MiddleParam {
        MiddleParam::try_from(String::from(value)).expect("Capability should be valid MiddleParam")
    }
}

impl From<Capability> for TrailingParam {
    fn from(value: Capability) -> TrailingParam {
        TrailingParam::from(MiddleParam::from(value))
    }
}

strserde!(Capability, "an IRCv3 capability name");

fn validate_capability(s: &str) -> Result<(), ParseCapabilityError> {
    if s.is_empty() {
        Err(ParseCapabilityError::Empty)
    } else if s.starts_with('-') {
        Err(ParseCapabilityError::BadStart)
    } else if s.contains(['\0', '\r', '\n', ' ', '=']) {
        Err(ParseCapabilityError::BadCharacter)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseCapabilityError {
    #[error("capabilities cannot be empty")]
    Empty,
    #[error("capabilities cannot start with '-'")]
    BadStart,
    #[error("capabilities cannot contain NUL, CR, LF, SPACE, or =")]
    BadCharacter,
}

// Based on the implementation of the `http::header` constants
macro_rules! impl_std {
    (
        $(
            $(#[$docs:meta])*
            ($konst:ident, $upcase:ident, $name:literal);
        )+
    ) => {
        #[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
        enum StandardCap {
            $($konst,)+
        }

        impl Capability {
            $(
                $(#[$docs])*
                pub const $upcase: Capability = Capability(InnerCap::Std(StandardCap::$konst));
            )+
        }

        impl StandardCap {
            #[inline]
            fn as_str(&self) -> &'static str {
                match self {
                    $(StandardCap::$konst => $name,)+
                }
            }

            fn from_str(name: &str) -> Option<StandardCap> {
                match name {
                    $($name => Some(StandardCap::$konst),)+
                    _ => None,
                }
            }
        }
    }
}

// See <https://ircv3.net/registry#capabilities> for the list of standard
// capabilities.
//
// Draft capabilities are not included here.
impl_std! {
    /// Notifies clients when other clients in common channels authenticate
    /// with or deauthenticate from their account (e.g. NickServ, SASL)
    ///
    /// Specification: <https://ircv3.net/specs/extensions/account-notify>
    (AccountNotify, ACCOUNT_NOTIFY, "account-notify");

    /// Attaches a tag containing the user’s account to every message they
    /// send
    ///
    /// Specification: <https://ircv3.net/specs/extensions/account-tag>
    (AccountTag, ACCOUNT_TAG, "account-tag");

    /// Notifies clients when other clients in common channels go away or come
    /// back
    ///
    /// Specification: <https://ircv3.net/specs/extensions/away-notify>
    (AwayNotify, AWAY_NOTIFY, "away-notify");

    /// Lets the server bundle common messages together, which lets clients be
    /// more intelligent about displaying them
    ///
    /// Specification: <https://ircv3.net/specs/extensions/batch>
    (Batch, BATCH, "batch");

    /// Notifies clients when client capabilities become available or are no
    /// longer available.
    ///
    /// Specification:
    /// <https://ircv3.net/specs/extensions/capability-negotiation#cap-notify>
    (CapNotify, CAP_NOTIFY, "cap-notify");

    /// Enables the CHGHOST message, which lets servers notify clients when
    /// another client's username and/or hostname changes
    ///
    /// Specification: <https://ircv3.net/specs/extensions/chghost>
    (ChgHost, CHGHOST, "chghost");

    /// Notifies clients when their PRIVMSG and NOTICEs are correctly received
    /// by the server
    ///
    /// Specification: <https://ircv3.net/specs/extensions/echo-message>
    (EchoMessage, ECHO_MESSAGE, "echo-message");

    /// Extends the JOIN message to include the account name of the joining
    /// client
    ///
    /// Specification: <https://ircv3.net/specs/extensions/extended-join>
    (ExtendedJoin, EXTENDED_JOIN, "extended-join");

    /// Extends the MONITOR spec to apply to more events
    ///
    /// Specification: <https://ircv3.net/specs/extensions/extended-monitor>
    (ExtendedMonitor, EXTENDED_MONITOR, "extended-monitor");

    /// Notifies clients when other clients are invited to common channels
    ///
    /// Specification: <https://ircv3.net/specs/extensions/invite-notify>
    (InviteNotify, INVITE_NOTIFY, "invite-notify");

    /// Allows clients to correlate requests with server responses
    ///
    /// Specification: <https://ircv3.net/specs/extensions/labeled-response>
    (LabeledResponse, LABELED_RESPONSE, "labeled-response");

    /// Allows clients and servers to use tags more broadly
    ///
    /// Specification: <https://ircv3.net/specs/extensions/message-tags>
    (MessageTags, MESSAGE_TAGS, "message-tags");

    /// Lets users request notifications for when clients become online /
    /// offline
    ///
    /// Specification: <https://ircv3.net/specs/extensions/monitor>
    (Monitor, MONITOR, "monitor");

    /// Makes the server send all prefixes in NAMES and WHO output, in order of
    /// rank from highest to lowest
    ///
    /// Specification: <https://ircv3.net/specs/extensions/multi-prefix>
    (MultiPrefix, MULTI_PREFIX, "multi-prefix");

    /// Disables implicit NAMES responses on JOIN
    ///
    /// Specification: <https://ircv3.net/specs/extensions/no-implicit-names>
    (NoImplicitNames, NO_IMPLICIT_NAMES, "no-implicit-names");

    /// Indicates support for SASL auth, a standardised way for clients to
    /// identify for an account
    ///
    /// Specifications:
    ///
    /// - <https://ircv3.net/specs/extensions/sasl-3.1>
    /// - <https://ircv3.net/specs/extensions/sasl-3.2>
    (Sasl, SASL,  "sasl");

    /// Lets clients show the actual time messages were received by the server
    ///
    /// Specification: <https://ircv3.net/specs/extensions/server-time>
    (ServerTime, SERVER_TIME, "server-time");

    /// Lets clients change their realname after connecting to the server
    ///
    /// Specification: <https://ircv3.net/specs/extensions/setname>
    (Setname, SETNAME, "setname");

    /// Allows servers to use standard replies more broadly
    ///
    /// Specification: <https://ircv3.net/specs/extensions/standard-replies>
    (StandardReplies, STANDARD_REPLIES, "standard-replies");

    /// Indicates support for the STARTTLS command, which lets clients upgrade
    /// their connection to use TLS encryption
    ///
    /// Specification: <https://ircv3.net/specs/deprecated/tls.html>
    (Tls, TLS, "tls");

    /// Extends the `RPL_NAMREPLY` message to contain the full nickmask
    /// (`nick!user@host`) of every user, rather than just the nickname
    ///
    /// Specification: <https://ircv3.net/specs/extensions/userhost-in-names>
    (UserhostInNames, USERHOST_IN_NAMES, "userhost-in-names");
}
