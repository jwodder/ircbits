mod ctcp;
mod handler_set;
mod ping;
pub use self::ctcp::*;
pub use self::handler_set::*;
pub use self::ping::*;
use irctext::{ClientMessage, Message};

/// A handler for automatically responding to messages received over IRC.
///
/// A handler is intended to be used as follows:
///
/// - First, call `get_client_messages()` and send any returned messages to the
///   server.
///
/// - Whenever a message is received from the server:
///
///     - Pass the message to `handle_message()`.
///
///     - Call `get_client_messages()` again and send any returned messages
///       back to the server.
///
///     - If `is_done()` returns `true`, discard the handler.
///
///     - If the call to `handle_message()` returned `false`, use the message
///       separately from the handler as you desire.
pub trait Handler {
    /// Returns outgoing messages to sent back to the server.
    ///
    /// The return value MAY be nonempty if called before `handle_message()`
    /// has been called, so users SHOULD call this method first before
    /// `handle_message()` and again after each call to `handle_message()`.
    ///
    /// If `is_done()` is true, this method SHOULD return an empty `Vec`.
    fn get_client_messages(&mut self) -> Vec<ClientMessage>;

    /// Handle an incoming message received from the server.  Returns `true` if
    /// the message should be considered "handled" by the handler and not be
    /// processed by any non-handlers.
    ///
    /// After calling this method, users SHOULD call `get_client_messages()`
    /// to receive any new outgoing messages from the handler.
    ///
    /// If `is_done()` is true, this method SHOULD be a no-op.
    fn handle_message(&mut self, msg: &Message) -> bool;

    /// Returns `true` when the handler has completed its tasks and is not
    /// interested in any more incoming messages.
    fn is_done(&self) -> bool;
}

impl<T: Handler + ?Sized> Handler for Box<T> {
    fn get_client_messages(&mut self) -> Vec<ClientMessage> {
        (**self).get_client_messages()
    }

    fn handle_message(&mut self, msg: &Message) -> bool {
        (**self).handle_message(msg)
    }

    fn is_done(&self) -> bool {
        (**self).is_done()
    }
}
