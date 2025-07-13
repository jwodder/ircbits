use super::Command;
use irctext::{
    ClientMessage, Message, Payload, Reply, ReplyParts, clientmsgs::List, types::Channel,
};
use std::time::Duration;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListCommand {
    outgoing: Vec<ClientMessage>,
    state: State,
}

impl ListCommand {
    pub fn new(msg: List) -> ListCommand {
        ListCommand {
            outgoing: vec![msg.into()],
            state: State::new(),
        }
    }
}

impl Command for ListCommand {
    type Output = Vec<ListEntry>;
    type Error = ListError;

    fn get_client_messages(&mut self) -> Vec<ClientMessage> {
        std::mem::take(&mut self.outgoing)
    }

    fn handle_message(&mut self, msg: &Message) -> bool {
        match &msg.payload {
            Payload::Reply(rpl) => {
                if rpl.is_error() && !matches!(rpl, Reply::NoMotd(_)) {
                    if !matches!(self.state, State::Listing(_)) {
                        return false;
                    }
                    let e = match rpl {
                        Reply::TryAgain(r) => ListError::TryAgain {
                            message: r.message().to_owned(),
                        },
                        Reply::InputTooLong(r) => ListError::InputTooLong {
                            message: r.message().to_string(),
                        },
                        Reply::UnknownCommand(r) => ListError::UnknownCommand {
                            command: r.command().to_string(),
                            message: r.message().to_string(),
                        },
                        Reply::NotRegistered(r) => ListError::NotRegistered {
                            message: r.message().to_string(),
                        },
                        unexpected => ListError::UnexpectedError {
                            code: unexpected.code(),
                            reply: msg.to_string(),
                        },
                    };
                    self.state = State::Done(Some(Err(e)));
                    true
                } else {
                    self.state.in_place(|state| state.handle_reply(rpl))
                }
            }
            Payload::ClientMessage(ClientMessage::Error(err)) => {
                self.state = State::Done(Some(Err(ListError::ErrorMessage {
                    reason: err.reason().to_string(),
                })));
                true
            }
            Payload::ClientMessage(_) => false,
        }
    }

    fn get_timeout(&mut self) -> Option<Duration> {
        None
    }

    fn handle_timeout(&mut self) {}

    fn is_done(&self) -> bool {
        matches!(self.state, State::Done(_))
    }

    fn get_output(&mut self) -> Result<Vec<ListEntry>, ListError> {
        if let State::Done(ref mut r) = self.state {
            r.take()
                .expect("get_output() should not be called more than once")
        } else {
            panic!("get_output() should only be called when is_done() is true");
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum State {
    Listing(Vec<ListEntry>),
    Done(Option<Result<Vec<ListEntry>, ListError>>),
    Void,
}

impl State {
    fn new() -> State {
        State::Listing(Vec::new())
    }
}

impl State {
    fn in_place<F>(&mut self, f: F) -> bool
    where
        F: FnOnce(Self) -> (State, bool),
    {
        let state = std::mem::replace(self, State::Void);
        let (st, b) = f(state);
        *self = st;
        b
    }

    fn handle_reply(self, rpl: &Reply) -> (State, bool) {
        match (self, rpl) {
            (st @ State::Listing(_), Reply::ListStart(_)) => (st, true),
            (State::Listing(mut chans), Reply::List(r)) => {
                let entry = ListEntry {
                    channel: r.channel().to_owned(),
                    clients: r.clients(),
                    topic: r.topic().to_owned(),
                };
                chans.push(entry);
                (State::Listing(chans), true)
            }
            (State::Listing(chans), Reply::ListEnd(_)) => (State::Done(Some(Ok(chans))), true),
            (State::Void, _) => panic!("handle_reply() called on Void list state"),
            (st, _) => (st, false),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ListEntry {
    pub channel: Channel,
    pub clients: u64,
    pub topic: String,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ListError {
    #[error("LIST failed: try again later: {message:?}")]
    TryAgain { message: String },
    #[error("LIST failed: registration required: {message:?}")]
    NotRegistered { message: String },
    #[error("LIST failed due to overly-long input line: {message:?}")]
    InputTooLong { message: String },
    #[error("LIST failed because server does not recognize {command:?} command: {message:?}")]
    UnknownCommand { command: String, message: String },
    #[error("server sent ERROR message during LIST reply: {reason:?}")]
    ErrorMessage { reason: String },
    #[error("LIST failed with unexpected error reply {code:03}: {reply:?}")]
    UnexpectedError { code: u16, reply: String },
    #[error(
        "LIST failed because server sent unexpected message: expecting {expecting}, got {msg:?}"
    )]
    Unexpected {
        expecting: &'static str,
        msg: String,
    },
}
