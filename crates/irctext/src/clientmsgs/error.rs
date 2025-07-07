use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    reason: FinalParam,
}

impl Error {
    pub fn new(reason: FinalParam) -> Error {
        Error { reason }
    }

    pub fn reason(&self) -> &FinalParam {
        &self.reason
    }

    pub fn into_reason(self) -> FinalParam {
        self.reason
    }
}

impl ClientMessageParts for Error {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Error,
            ParameterList::builder().with_final(self.reason),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("ERROR :{}", self.reason)
    }
}

impl From<Error> for Message {
    fn from(value: Error) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Error> for RawMessage {
    fn from(value: Error) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Error {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Error, ClientMessageError> {
        let (reason,) = params.try_into()?;
        Ok(Error { reason })
    }
}
