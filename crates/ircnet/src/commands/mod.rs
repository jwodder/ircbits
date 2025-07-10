use irctext::{ClientMessage, Message};
use std::time::Duration;

/// A trait for sending messages to an IRC server and handling the replies.
///
/// A `Command` is intended to be used as follows:
///
/// - First, call `get_client_messages()` and send any returned messages to the
///   server.
///
/// - Call `get_timeout()`; if it returns non-`None`, schedule a timeout after
///   the given delay.
///
/// - While receiving messages from the server and awaiting any timeouts:
///
///     - If a message is received, pass it to `handle_message()`, and then
///       call `get_client_messages()` again and send any returned messages.
///       Also call `get_timeout()` again; if it returns non-`None`, update the
///       scheduled timeout to the new delay.
///
///     - If a timeout occurs, call `handle_timeout()`, and then call
///       `get_client_messages()` and `get_timeout()` as above.
///
///     - After performing either of the above steps, if `is_done()` returns
///       `true`, call `get_output()` to get the result of the command, and
///       then discard the command.
pub trait Command {
    /// Information returned by the command upon completion, whether successful
    /// or not
    type Output;

    /// Returns outgoing messages to send back to the server.
    ///
    /// Users SHOULD call this method first before `handle_message()` and again
    /// after each call to `handle_message()` or `handle_timeout()`.
    ///
    /// If `is_done()` is true, this method SHOULD return an empty `Vec`.
    fn get_client_messages(&mut self) -> Vec<ClientMessage>;

    /// Handle an incoming message received from the server.  Returns `true` if
    /// the message should be considered "handled" by the command and not to be
    /// returned to the calling context.
    ///
    /// After calling this method, users SHOULD call `get_client_messages()`
    /// and `get_timeout()` to receive any updated outgoing events from the
    /// command.
    ///
    /// If `is_done()` is true, this method SHOULD be a no-op.
    fn handle_message(&mut self, msg: &Message) -> bool;

    /// If the command wishes for the caller to schedule a timeout, this method
    /// will return the duration until that timeout.  Once the timeout occurs,
    /// `handle_timeout()` should be called.
    ///
    /// If a later call to this method returns a new duration, the user should
    /// cancel/discard the previously-scheduled timeout and schedule a new
    /// timeout in its place with the new duration.
    ///
    /// Users SHOULD call this method first before `handle_message()` and again
    /// after each call to `handle_message()` or `handle_timeout()`.
    ///
    /// If `is_done()` is true, this method SHOULD return `None`.
    fn get_timeout(&mut self) -> Option<Duration>;

    /// Called after a timeout specified by `get_timeout()` occurs.
    ///
    /// If there is no active timeout — i.e., if no call to `get_timeout()`
    /// returned `Some` since either command creation or the previous call to
    /// `handle_timeout()` — this method SHOULD be a no-op.
    ///
    /// If `is_done()` is true, this method SHOULD be a no-op.
    fn handle_timeout(&mut self);

    /// Returns `true` when the command has completed its tasks (whether
    /// successfully or not) and is not interested in any more incoming
    /// messages or timeouts.
    fn is_done(&self) -> bool;

    /// Returns the result of the command.  MUST only be called once
    /// `is_done()` returns true.
    ///
    /// If `is_done()` is not true, this method MAY panic.
    fn get_output(&mut self) -> Self::Output;
}
