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
/// - The constructor for a `SaslFlow` value should return the new object
///   alongside an `Authenticate` message.  Send this message to the server.
///
/// - Whenever a message is received from the server:
///
///     - If the message is an `AUTHENTICATE` command, pass it to
///       `handle_message()`.
///
///         - If `Ok(msgs)` is returned, send `msgs` to the server, then call
///           `is_done()`.  If it returns `true`, the `SaslFlow` has done all
///           it can, and the object should be discarded without calling any
///           further methods.  Success of the SASL operation should then be
///           judged based on the replies returned by the server.
///
///         - If an error is returned, then SASL has failed and the `SaslFlow`
///           object should be discarded without calling any further methods on
///           it.
///
///     - If the message is anything else, it should be handled outside of the
///       `SaslFlow`.  Error replies relating to the SASL process should result
///       in the `SaslFlow` object being discarded.  Client messages other than
///       `Authenticate` should not normally be received while SASL
///       authentication is in progress.
#[enum_dispatch]
pub trait SaslFlow {
    fn handle_message(&mut self, msg: &Authenticate) -> Result<Vec<Authenticate>, SaslError>;
    fn is_done(&self) -> bool;
}

#[enum_dispatch(SaslFlow)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SaslMachine {
    Plain(PlainSasl),
    Scram(ScramSasl),
}

#[derive(
    strum::AsRefStr,
    Clone,
    Copy,
    Debug,
    strum::Display,
    strum::EnumIter,
    strum::EnumString,
    Eq,
    Hash,
    PartialEq,
)]
pub enum SaslMechanism {
    #[strum(to_string = "PLAIN")]
    Plain,
    #[strum(to_string = "SCRAM-SHA-1")]
    ScramSha1,
    #[strum(to_string = "SCRAM-SHA-256")]
    ScramSha256,
    #[strum(to_string = "SCRAM-SHA-512")]
    ScramSha512,
}

impl SaslMechanism {
    pub fn iter() -> SaslMechanismIter {
        <SaslMechanism as strum::IntoEnumIterator>::iter()
    }

    pub fn new_flow(
        self,
        nickname: &Nickname,
        password: &str,
    ) -> Result<(SaslMachine, Authenticate), SaslError> {
        match self {
            SaslMechanism::Plain => {
                let (machine, msg1) = PlainSasl::new(nickname, password);
                Ok((machine.into(), msg1))
            }
            SaslMechanism::ScramSha1 => {
                let (machine, msg1) = ScramSasl::new(nickname, password, HashAlgo::Sha1)?;
                Ok((machine.into(), msg1))
            }
            SaslMechanism::ScramSha256 => {
                let (machine, msg1) = ScramSasl::new(nickname, password, HashAlgo::Sha256)?;
                Ok((machine.into(), msg1))
            }
            SaslMechanism::ScramSha512 => {
                let (machine, msg1) = ScramSasl::new(nickname, password, HashAlgo::Sha512)?;
                Ok((machine.into(), msg1))
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
