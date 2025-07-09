use crate::{
    ClientMessage, ClientMessageError, ClientMessageParts, Command, ParameterList,
    ParseRawMessageError, RawMessage, Reply, ReplyError, ReplyParts, Source, TryFromStringError,
};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub source: Option<Source>,
    pub payload: Payload,
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
