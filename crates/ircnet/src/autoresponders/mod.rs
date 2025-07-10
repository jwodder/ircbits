mod ctcp;
mod ping;
mod responderset;
pub use self::ctcp::*;
pub use self::ping::*;
pub use self::responderset::*;
use irctext::{ClientMessage, Message};

/// A handler/observer for automatically responding to messages received over IRC
///
/// An `AutoResponder` is intended to be used as follows whenever a message is
/// received from an IRC server:
///
/// - Pass the message to `handle_message()`.
///
/// - Call `get_client_messages()` and send any returned messages back to the
///   server.
///
/// - If `is_done()` returns `true`, discard the autoresponder.
///
/// - If the call to `handle_message()` returned `false`, use the message
///   separately from the autoresponder as you desire.
pub trait AutoResponder {
    /// Returns outgoing messages to sent back to the server.
    ///
    /// Users SHOULD call this method after each call to `handle_message()`.
    ///
    /// If `is_done()` is true, this method SHOULD return an empty `Vec`.
    fn get_client_messages(&mut self) -> Vec<ClientMessage>;

    /// Handle an incoming message received from the server.  Returns `true` if
    /// the message should be considered "handled" by the autoresponder and not to be
    /// processed by any non-autoresponders.
    ///
    /// After calling this method, users SHOULD call `get_client_messages()`
    /// to receive any new outgoing messages from the autoresponder.
    ///
    /// If `is_done()` is true, this method SHOULD be a no-op.
    fn handle_message(&mut self, msg: &Message) -> bool;

    /// Returns `true` when the autoresponder has completed its tasks and is
    /// not interested in any more incoming messages.
    fn is_done(&self) -> bool;
}

impl<T: AutoResponder + ?Sized> AutoResponder for Box<T> {
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
