use super::lines::{LinesCodec, LinesCodecError};
use bytes::BytesMut;
use irctext::{ClientMessageParts, Message, ParseMessageError, TryFromStringError};
use std::io;
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MessageCodec(LinesCodec);

impl MessageCodec {
    pub fn new() -> MessageCodec {
        MessageCodec(LinesCodec::new())
    }

    pub fn new_with_max_length(max_length: usize) -> MessageCodec {
        MessageCodec(LinesCodec::new_with_max_length(max_length))
    }
}

impl Decoder for MessageCodec {
    type Item = Message;
    type Error = MessageCodecError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Message>, MessageCodecError> {
        match self.0.decode(buf) {
            Ok(Some(line)) => Ok(Some(Message::try_from(line)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<Message>, MessageCodecError> {
        match self.0.decode_eof(buf) {
            Ok(Some(line)) => Ok(Some(Message::try_from(line)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

impl<T: ClientMessageParts> Encoder<T> for MessageCodec {
    type Error = MessageCodecError;

    fn encode(&mut self, msg: T, buf: &mut BytesMut) -> Result<(), MessageCodecError> {
        self.0.encode(msg.to_irc_line(), buf).map_err(Into::into)
    }
}

impl From<LinesCodec> for MessageCodec {
    fn from(value: LinesCodec) -> MessageCodec {
        MessageCodec(value)
    }
}

impl Default for MessageCodec {
    fn default() -> MessageCodec {
        MessageCodec::new()
    }
}

#[derive(Debug, Error)]
pub enum MessageCodecError {
    #[error("maximum incoming line length exceeded")]
    MaxLineLengthExceeded,

    #[error("I/O error communicating with server")]
    Io(#[from] io::Error),

    #[error("failed to parse incoming message")]
    Parse(#[from] TryFromStringError<ParseMessageError>),
}

impl From<LinesCodecError> for MessageCodecError {
    fn from(e: LinesCodecError) -> MessageCodecError {
        match e {
            LinesCodecError::MaxLineLengthExceeded => MessageCodecError::MaxLineLengthExceeded,
            LinesCodecError::Io(inner) => MessageCodecError::Io(inner),
        }
    }
}
