use crate::{
    ClientMessage, ClientMessageError, ClientMessageParts, Command, ParameterList,
    ParseRawMessageError, RawMessage, Reply, ReplyError, ReplyParts, Source, TryFromStringError,
};
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub source: Option<Source>,
    pub payload: Payload,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(source) = self.source.as_ref() {
            write!(f, ":{source} ")?;
        }
        match &self.payload {
            Payload::ClientMessage(msg) => write!(f, "{}", msg.to_irc_line())?,
            Payload::Reply(r) => {
                write!(f, "{:03}", r.code())?;
                let params = r.parameters();
                if !params.is_empty() {
                    write!(f, "{params}")?;
                }
            }
        }
        Ok(())
    }
}

impl std::str::FromStr for Message {
    type Err = ParseMessageError;

    fn from_str(s: &str) -> Result<Message, ParseMessageError> {
        Message::try_from(s.parse::<RawMessage>()?).map_err(Into::into)
    }
}

impl TryFrom<String> for Message {
    type Error = TryFromStringError<ParseMessageError>;

    fn try_from(string: String) -> Result<Message, TryFromStringError<ParseMessageError>> {
        match string.parse() {
            Ok(msg) => Ok(msg),
            Err(inner) => Err(TryFromStringError { inner, string }),
        }
    }
}

impl From<Payload> for Message {
    fn from(payload: Payload) -> Message {
        Message {
            source: None,
            payload,
        }
    }
}

impl TryFrom<RawMessage> for Message {
    type Error = MessageError;

    fn try_from(msg: RawMessage) -> Result<Message, MessageError> {
        let source = msg.source;
        let payload = Payload::from_parts(msg.command, msg.parameters)?;
        Ok(Message { source, payload })
    }
}

impl From<Message> for RawMessage {
    fn from(msg: Message) -> RawMessage {
        let source = msg.source;
        let (command, parameters) = msg.payload.into_parts();
        RawMessage {
            source,
            command,
            parameters,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Payload {
    ClientMessage(ClientMessage),
    Reply(Reply),
}

impl Payload {
    pub fn is_client_message(&self) -> bool {
        matches!(self, Payload::ClientMessage(_))
    }

    pub fn is_reply(&self) -> bool {
        matches!(self, Payload::Reply(_))
    }
}

impl Payload {
    pub fn from_parts(cmd: Command, params: ParameterList) -> Result<Payload, MessageError> {
        match cmd {
            Command::Verb(v) => Ok(Payload::ClientMessage(ClientMessage::from_parts(
                v, params,
            )?)),
            Command::Reply(code) => Ok(Payload::Reply(Reply::from_parts(code, params)?)),
        }
    }

    pub fn into_parts(self) -> (Command, ParameterList) {
        match self {
            Payload::ClientMessage(msg) => {
                let (verb, params) = msg.into_parts();
                (Command::Verb(verb), params)
            }
            Payload::Reply(r) => {
                let (code, params) = r.into_parts();
                (Command::Reply(code), params)
            }
        }
    }
}

impl From<ClientMessage> for Payload {
    fn from(value: ClientMessage) -> Payload {
        Payload::ClientMessage(value)
    }
}

impl From<Reply> for Payload {
    fn from(value: Reply) -> Payload {
        Payload::Reply(value)
    }
}

impl PartialEq<ClientMessage> for Payload {
    fn eq(&self, other: &ClientMessage) -> bool {
        matches!(self, Payload::ClientMessage(msg) if msg == other)
    }
}

impl PartialEq<Reply> for Payload {
    fn eq(&self, other: &Reply) -> bool {
        matches!(self, Payload::Reply(r) if r == other)
    }
}

#[derive(Debug, Error)]
pub enum MessageError {
    #[error(transparent)]
    ClientMessage(#[from] ClientMessageError),
    #[error(transparent)]
    Reply(#[from] ReplyError),
}

#[derive(Debug, Error)]
pub enum ParseMessageError {
    #[error(transparent)]
    ParseRaw(#[from] ParseRawMessageError),
    #[error(transparent)]
    Convert(#[from] MessageError),
}
