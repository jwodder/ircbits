use super::lines::{LinesCodec, LinesCodecError};
use bytes::BytesMut;
use irctext::{ParseRawMessageError, RawMessage, TryFromStringError};
use std::io;
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RawMessageCodec(LinesCodec);

impl RawMessageCodec {
    pub fn new() -> RawMessageCodec {
        RawMessageCodec(LinesCodec::new())
    }

    pub fn new_with_max_length(max_length: usize) -> RawMessageCodec {
        RawMessageCodec(LinesCodec::new_with_max_length(max_length))
    }
}

impl Decoder for RawMessageCodec {
    type Item = RawMessage;
    type Error = RawMessageCodecError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<RawMessage>, RawMessageCodecError> {
        match self.0.decode(buf) {
            Ok(Some(line)) => Ok(Some(RawMessage::try_from(line)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn decode_eof(
        &mut self,
        buf: &mut BytesMut,
    ) -> Result<Option<RawMessage>, RawMessageCodecError> {
        match self.0.decode_eof(buf) {
            Ok(Some(line)) => Ok(Some(RawMessage::try_from(line)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

impl Encoder<RawMessage> for RawMessageCodec {
    type Error = RawMessageCodecError;

    fn encode(&mut self, msg: RawMessage, buf: &mut BytesMut) -> Result<(), RawMessageCodecError> {
        self.0.encode(msg.to_string(), buf).map_err(Into::into)
    }
}

impl From<LinesCodec> for RawMessageCodec {
    fn from(value: LinesCodec) -> RawMessageCodec {
        RawMessageCodec(value)
    }
}

impl Default for RawMessageCodec {
    fn default() -> RawMessageCodec {
        RawMessageCodec::new()
    }
}

#[derive(Debug, Error)]
pub enum RawMessageCodecError {
    #[error("maximum incoming line length exceeded")]
    MaxLineLengthExceeded,

    #[error("I/O error communicating with server")]
    Io(#[from] io::Error),

    #[error("failed to parse incoming message")]
    Parse(#[from] TryFromStringError<ParseRawMessageError>),
}

impl From<LinesCodecError> for RawMessageCodecError {
    fn from(e: LinesCodecError) -> RawMessageCodecError {
        match e {
            LinesCodecError::MaxLineLengthExceeded => RawMessageCodecError::MaxLineLengthExceeded,
            LinesCodecError::Io(inner) => RawMessageCodecError::Io(inner),
        }
    }
}
