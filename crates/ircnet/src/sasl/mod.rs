mod plain;
mod scram;
pub use self::plain::PlainSasl;
pub use self::scram::*;
use enum_dispatch::enum_dispatch;
use irctext::{clientmsgs::Authenticate, types::Nickname};
use thiserror::Error;

/// A trait for sans IO state machines for authenticating with an IRC server
/// via SASL.
///
/// A `SaslFlow` is intended to be used as follows:
///
/// - First, call `get_output()` and send any returned messages to the server.
///
/// - Whenever a message is received from the server:
///
///     - If the message is an `AUTHENTICATE` command, pass it to
///       `handle_message()`.  If an error is returned, then SASL has failed
///       and the `SaslFlow` object should be discarded without calling any
///       further methods on it.
///
///     - If the message is anything else, it should be handled outside of the
///       `SaslFlow`.  Error replies relating to the SASL process should result
///       in the `SaslFlow` object being discarded.  Client messages other than
///       `Authenticate` should not normally be received while SASL
///       authentication is in progress.
///
/// - After each call to `handle_message()`, call `get_output()` again and send
///   any returned messages to the server.
///
/// - After each call to `get_output()` and sending the returned messages
///   (including the initial call), call `is_done()`.  If it returns `true`,
///   the `SaslFlow` has done all it can, and the object should be discarded
///   without calling any further methods.  Success of the SASL operation
///   should then be judged based on the replies returned by the server.
#[enum_dispatch]
pub trait SaslFlow {
    fn handle_message(&mut self, msg: Authenticate) -> Result<(), SaslError>;
    fn get_output(&mut self) -> Vec<Authenticate>;
    fn is_done(&self) -> bool;
}

#[enum_dispatch(SaslFlow)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SaslMachine {
    Plain(PlainSasl),
    Scram(ScramSasl),
}

#[derive(
    strum::AsRefStr, Clone, Copy, Debug, strum::Display, strum::EnumString, Eq, Hash, PartialEq,
)]
pub enum SaslMechanism {
    #[strum(to_string = "PLAIN")]
    Plain,
    #[strum(to_string = "SCRAM-SHA-1")]
    ScramSha1,
    #[strum(to_string = "SCRAM-SHA-512")]
    ScramSha512,
}

impl SaslMechanism {
    pub fn new_flow(self, nickname: &Nickname, password: &str) -> Result<SaslMachine, SaslError> {
        match self {
            SaslMechanism::Plain => Ok(PlainSasl::new(nickname, password).into()),
            SaslMechanism::ScramSha1 => {
                Ok(ScramSasl::new(nickname, password, HashAlgo::Sha1)?.into())
            }
            SaslMechanism::ScramSha512 => {
                Ok(ScramSasl::new(nickname, password, HashAlgo::Sha512)?.into())
            }
        }
    }
}

pub type ParseSaslMechanismError = strum::ParseError;

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl serde::Serialize for SaslMechanism {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> serde::Deserialize<'de> for SaslMechanism {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = SaslMechanism;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a supported SASL mechanism")
            }

            fn visit_str<E>(self, input: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                input
                    .parse::<SaslMechanism>()
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(input), &self))
            }
        }

        deserializer.deserialize_string(Visitor)
    }
}

#[derive(Debug, Error)]
pub enum SaslError {
    #[error("server sent unexpected message: expecting {expecting}, got {msg:?}")]
    Unexpected {
        expecting: &'static str,
        msg: String,
    },
    #[error("username preparation failed")]
    PrepareUsername(#[from] PrepareUsernameError),
    #[error("password preparation failed")]
    PreparePassword(#[source] stringprep::Error),
    #[error("failed to decode base64 payload from server")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("failed to decode message from server as UTF-8")]
    Utf8Decode(#[from] std::str::Utf8Error),
    #[error("nonce returned by server did not start with our nonce")]
    Nonce,
    #[error("mismatch between signatures computed by client and server")]
    Signature,
    #[error("server returned error: {0:?}")]
    Server(String),
    #[error("failed to parse message from server")]
    Parse,
}
