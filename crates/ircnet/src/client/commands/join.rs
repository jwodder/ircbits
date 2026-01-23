use super::Command;
use irctext::{
    ClientMessage, ClientSource, Message, Payload, Reply, ReplyParts,
    clientmsgs::Join,
    types::{Channel, ChannelMembership, ChannelStatus, Key, Nickname},
};
use std::time::Duration;
use thiserror::Error;

/// How long to wait for an optional `RPL_CHANNELURL` (328) message after
/// receiving the list of names
const URL_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JoinCommand {
    outgoing: Vec<ClientMessage>,
    state: State,
}

impl JoinCommand {
    pub fn new(channel: Channel) -> JoinCommand {
        JoinCommand {
            outgoing: vec![Join::new(channel).into()],
            state: State::Start,
        }
    }

    pub fn new_with_key(channel: Channel, key: Key) -> JoinCommand {
        JoinCommand {
            outgoing: vec![Join::new_with_key(channel, key).into()],
            state: State::Start,
        }
    }
}

// Order of replies on sucessful JOIN:
//  - JOIN
//  - One of:
//     - RPL_TOPIC (332) + optional RPL_TOPICWHOTIME (333)
//     - no replies
//  - one or more RPL_NAMREPLY (353)
//  - RPL_ENDOFNAMES (366)
//  - optional: RPL_CHANNELURL (328)

// Possible error replies:
//  - ERROR message
//  - ERR_NOSUCHCHANNEL (403)
//  - ERR_TOOMANYCHANNELS (405)
//  - ERR_CHANNELISFULL (471)
//  - ERR_INVITEONLYCHAN (473)
//  - ERR_BANNEDFROMCHAN (474)
//  - ERR_BADCHANNELKEY (475)
//  - ERR_BADCHANMASK (476)
//  - RPL_TRYAGAIN (263)
//  - ERR_INPUTTOOLONG (417)
//  - ERR_UNKNOWNCOMMAND (421)
//  - ERR_NOTREGISTERED (451)
//  - ERR_NEEDMOREPARAMS (461) ?

impl Command for JoinCommand {
    type Output = JoinOutput;
    type Error = JoinError;

    fn get_client_messages(&mut self) -> Vec<ClientMessage> {
        std::mem::take(&mut self.outgoing)
    }

    fn handle_message(&mut self, msg: &Message) -> bool {
        match &msg.payload {
            Payload::Reply(rpl) => {
                if rpl.is_error() && !matches!(rpl, Reply::NoMotd(_)) {
                    if self.state != State::Start {
                        return false;
                    }
                    let e = match rpl {
                        Reply::NoSuchChannel(r) => JoinError::NoSuchChannel {
                            message: r.message().to_owned(),
                        },
                        Reply::TooManyChannels(r) => JoinError::TooManyChannels {
                            message: r.message().to_owned(),
                        },
                        Reply::ChannelIsFull(r) => JoinError::ChannelIsFull {
                            message: r.message().to_owned(),
                        },
                        Reply::InviteOnlyChan(r) => JoinError::InviteOnly {
                            message: r.message().to_owned(),
                        },
                        Reply::BannedFromChan(r) => JoinError::Banned {
                            message: r.message().to_owned(),
                        },
                        Reply::BadChannelKey(r) => JoinError::BadChannelKey {
                            message: r.message().to_owned(),
                        },
                        Reply::TryAgain(r) => JoinError::TryAgain {
                            message: r.message().to_owned(),
                        },
                        Reply::InputTooLong(r) => JoinError::InputTooLong {
                            message: r.message().to_string(),
                        },
                        Reply::UnknownCommand(r) => JoinError::UnknownCommand {
                            command: r.command().to_string(),
                            message: r.message().to_string(),
                        },
                        Reply::NotRegistered(r) => JoinError::NotRegistered {
                            message: r.message().to_string(),
                        },
                        unexpected => JoinError::UnexpectedError {
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
                self.state = State::Done(Some(Err(JoinError::ErrorMessage {
                    reason: err.reason().to_string(),
                })));
                true
            }
            Payload::ClientMessage(ClientMessage::Join(_)) => {
                self.state.in_place(State::handle_join)
            }
            Payload::ClientMessage(_) => false,
        }
    }

    fn get_timeout(&mut self) -> Option<Duration> {
        if let State::AwaitingUrl {
            ref mut timeout, ..
        } = self.state
        {
            timeout.take()
        } else {
            None
        }
    }

    fn handle_timeout(&mut self) {
        let state = std::mem::replace(&mut self.state, State::Void);
        self.state = match state {
            State::AwaitingUrl {
                timeout: None,
                output,
            } => State::Done(Some(Ok(output))),
            other => other,
        };
    }

    fn is_done(&self) -> bool {
        matches!(self.state, State::Done(_))
    }

    fn get_output(&mut self) -> Result<JoinOutput, JoinError> {
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
    Start,
    GotJoin,
    GotTopic {
        topic: String,
    },
    GotTopicWho {
        topic: String,
        topic_setter: ClientSource,
        topic_set_at: u64,
    },
    GotNamReply(JoinOutput),
    AwaitingUrl {
        output: JoinOutput,
        timeout: Option<Duration>,
    },
    Done(Option<Result<JoinOutput, JoinError>>),
    Void,
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
            (State::GotJoin, Reply::Topic(r)) => (
                State::GotTopic {
                    topic: r.topic().to_owned(),
                },
                true,
            ),
            (State::GotTopic { topic }, Reply::TopicWhoTime(r)) => (
                State::GotTopicWho {
                    topic,
                    topic_setter: r.user().clone(),
                    topic_set_at: r.setat(),
                },
                true,
            ),
            (State::GotJoin, Reply::NamReply(r)) => (
                State::GotNamReply(JoinOutput {
                    channel: r.channel().to_owned(),
                    topic: None,
                    topic_set_by: None,
                    topic_set_at: None,
                    channel_status: r.channel_status(),
                    users: r.clients().to_vec(),
                    url: None,
                }),
                true,
            ),
            (State::GotTopic { topic }, Reply::NamReply(r)) => (
                State::GotNamReply(JoinOutput {
                    channel: r.channel().to_owned(),
                    topic: Some(topic),
                    topic_set_by: None,
                    topic_set_at: None,
                    channel_status: r.channel_status(),
                    users: r.clients().to_vec(),
                    url: None,
                }),
                true,
            ),
            (
                State::GotTopicWho {
                    topic,
                    topic_setter,
                    topic_set_at,
                },
                Reply::NamReply(r),
            ) => (
                State::GotNamReply(JoinOutput {
                    channel: r.channel().to_owned(),
                    topic: Some(topic),
                    topic_set_by: Some(topic_setter),
                    topic_set_at: Some(topic_set_at),
                    channel_status: r.channel_status(),
                    users: r.clients().to_vec(),
                    url: None,
                }),
                true,
            ),
            (State::GotNamReply(mut output), Reply::NamReply(r)) => {
                output.users.extend(r.clients().to_vec());
                (State::GotNamReply(output), true)
            }
            (State::GotNamReply(output), Reply::EndOfNames(_)) => (
                State::AwaitingUrl {
                    output,
                    timeout: Some(URL_TIMEOUT),
                },
                true,
            ),
            (State::AwaitingUrl { mut output, .. }, Reply::ChannelUrl(r)) => {
                output.url = Some(r.url().to_owned());
                (State::Done(Some(Ok(output))), true)
            }
            (State::Void, _) => panic!("handle_reply() called on Void join state"),
            (st, _) => (st, false),
        }
    }

    fn handle_join(self) -> (State, bool) {
        match self {
            State::Start => (State::GotJoin, true),
            State::Void => panic!("handle_join() called on Void join state"),
            st => (st, false),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JoinOutput {
    /// The name of the channel as given in the `RPL_NAMREPLY` messages
    pub channel: Channel,

    /// The channel's topic, or `None` if no topic is set
    pub topic: Option<String>,

    /// The user who set the current topic, or `None` if not reported
    pub topic_set_by: Option<ClientSource>,

    /// The time at which the current topic was set as a UNIX timestamp, or
    /// `None` if not reported
    pub topic_set_at: Option<u64>,

    /// The channel's status
    pub channel_status: ChannelStatus,

    /// The users currently joined to the channel along with their membership
    /// statuses therein
    pub users: Vec<(Option<ChannelMembership>, Nickname)>,

    /// The URL associated with the channel, if any
    pub url: Option<String>,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum JoinError {
    #[error("JOIN failed: no such channel: {message:?}")]
    NoSuchChannel { message: String },
    #[error("JOIN failed: you have joined too many channels: {message:?}")]
    TooManyChannels { message: String },
    #[error("JOIN failed: channel is full: {message:?}")]
    ChannelIsFull { message: String },
    #[error("JOIN failed: channel is invite-only: {message:?}")]
    InviteOnly { message: String },
    #[error("JOIN failed: you are banned from the channel: {message:?}")]
    Banned { message: String },
    #[error("JOIN failed: bad channel key: {message:?}")]
    BadChannelKey { message: String },
    #[error("JOIN failed: try again later: {message:?}")]
    TryAgain { message: String },
    #[error("JOIN failed: registration required: {message:?}")]
    NotRegistered { message: String },
    #[error("JOIN failed due to overly-long input line: {message:?}")]
    InputTooLong { message: String },
    #[error("JOIN failed because server does not recognize {command:?} command: {message:?}")]
    UnknownCommand { command: String, message: String },
    #[error("server sent ERROR message during JOIN: {reason:?}")]
    ErrorMessage { reason: String },
    #[error("JOIN failed with unexpected error reply {code:03}: {reply:?}")]
    UnexpectedError { code: u16, reply: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use irctext::ClientMessageParts;

    #[test]
    fn basic_join() {
        let channel = "#test".parse::<Channel>().unwrap();
        let mut cmd = JoinCommand::new(channel);
        let outgoing = cmd
            .get_client_messages()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["JOIN #test"]);
        let received = [
            ":mynick!~myuser@myhost.nil JOIN #test",
            ":irc.example.com 332 mynick #test :A test channel",
            ":irc.example.com 333 mynick #test chanop!~channer@elsewhere.test 1757535834",
            ":irc.example.com 353 mynick = #test :mynick ~TestFather @testmod +speaker luser",
            ":irc.example.com 353 mynick = #test :SomeOtherUser visitor im-lost tester +testbot eg.zample",
            ":irc.example.com 366 mynick #test :End of /NAMES list.",
        ];
        for m in received {
            let msg = m.parse::<Message>().unwrap();
            assert!(cmd.handle_message(&msg));
            assert!(cmd.get_client_messages().is_empty());
            assert!(!cmd.is_done());
        }
        assert!(cmd.get_timeout().is_some());
        cmd.handle_timeout();
        assert!(cmd.get_client_messages().is_empty());
        assert!(cmd.is_done());
        let output = cmd.get_output().unwrap();
        pretty_assertions::assert_eq!(
            output,
            JoinOutput {
                channel: "#test".parse::<Channel>().unwrap(),
                topic: Some(String::from("A test channel")),
                topic_set_by: Some(
                    "chanop!~channer@elsewhere.test"
                        .parse::<ClientSource>()
                        .unwrap()
                ),
                topic_set_at: Some(1757535834),
                channel_status: ChannelStatus::Public,
                users: vec![
                    (None, "mynick".parse::<Nickname>().unwrap()),
                    (
                        Some(ChannelMembership::Founder),
                        "TestFather".parse::<Nickname>().unwrap()
                    ),
                    (
                        Some(ChannelMembership::Operator),
                        "testmod".parse::<Nickname>().unwrap()
                    ),
                    (
                        Some(ChannelMembership::Voiced),
                        "speaker".parse::<Nickname>().unwrap()
                    ),
                    (None, "luser".parse::<Nickname>().unwrap()),
                    (None, "SomeOtherUser".parse::<Nickname>().unwrap()),
                    (None, "visitor".parse::<Nickname>().unwrap()),
                    (None, "im-lost".parse::<Nickname>().unwrap()),
                    (None, "tester".parse::<Nickname>().unwrap()),
                    (
                        Some(ChannelMembership::Voiced),
                        "testbot".parse::<Nickname>().unwrap()
                    ),
                    (None, "eg.zample".parse::<Nickname>().unwrap()),
                ],
                url: None,
            }
        );
    }

    #[test]
    fn join_channel_with_url() {
        let channel = "#test".parse::<Channel>().unwrap();
        let mut cmd = JoinCommand::new(channel);
        let outgoing = cmd
            .get_client_messages()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["JOIN #test"]);
        let received = [
            ":mynick!~myuser@myhost.nil JOIN #test",
            ":irc.example.com 332 mynick #test :A test channel",
            ":irc.example.com 333 mynick #test chanop!~channer@elsewhere.test 1757535834",
            ":irc.example.com 353 mynick = #test :mynick ~TestFather @testmod +speaker luser",
            ":irc.example.com 353 mynick = #test :SomeOtherUser visitor im-lost tester +testbot eg.zample",
            ":irc.example.com 366 mynick #test :End of /NAMES list.",
        ];
        for m in received {
            let msg = m.parse::<Message>().unwrap();
            assert!(cmd.handle_message(&msg));
            assert!(cmd.get_client_messages().is_empty());
            assert!(!cmd.is_done());
        }
        assert!(cmd.get_timeout().is_some());
        let m = ":services. 328 mynick #test :test.example.com";
        let msg = m.parse::<Message>().unwrap();
        assert!(cmd.handle_message(&msg));
        assert!(cmd.get_client_messages().is_empty());
        assert!(cmd.is_done());
        let output = cmd.get_output().unwrap();
        pretty_assertions::assert_eq!(
            output,
            JoinOutput {
                channel: "#test".parse::<Channel>().unwrap(),
                topic: Some(String::from("A test channel")),
                topic_set_by: Some(
                    "chanop!~channer@elsewhere.test"
                        .parse::<ClientSource>()
                        .unwrap()
                ),
                topic_set_at: Some(1757535834),
                channel_status: ChannelStatus::Public,
                users: vec![
                    (None, "mynick".parse::<Nickname>().unwrap()),
                    (
                        Some(ChannelMembership::Founder),
                        "TestFather".parse::<Nickname>().unwrap()
                    ),
                    (
                        Some(ChannelMembership::Operator),
                        "testmod".parse::<Nickname>().unwrap()
                    ),
                    (
                        Some(ChannelMembership::Voiced),
                        "speaker".parse::<Nickname>().unwrap()
                    ),
                    (None, "luser".parse::<Nickname>().unwrap()),
                    (None, "SomeOtherUser".parse::<Nickname>().unwrap()),
                    (None, "visitor".parse::<Nickname>().unwrap()),
                    (None, "im-lost".parse::<Nickname>().unwrap()),
                    (None, "tester".parse::<Nickname>().unwrap()),
                    (
                        Some(ChannelMembership::Voiced),
                        "testbot".parse::<Nickname>().unwrap()
                    ),
                    (None, "eg.zample".parse::<Nickname>().unwrap()),
                ],
                url: Some(String::from("test.example.com")),
            }
        );
    }
}
