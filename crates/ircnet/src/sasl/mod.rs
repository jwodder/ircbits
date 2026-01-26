mod plain;
pub use self::plain::PlainSasl;
use irctext::clientmsgs::Authenticate;
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
pub trait SaslFlow {
    fn handle_message(&mut self, msg: Authenticate) -> Result<(), SaslError>;
    fn get_output(&mut self) -> Vec<Authenticate>;
    fn is_done(&self) -> bool;
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SaslError {
    #[error(
        "SASL failed because server sent unexpected message: expecting {expecting}, got {msg:?}"
    )]
    Unexpected {
        expecting: &'static str,
        msg: String,
    },
}
