use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, MiddleParam, ParameterList, RawMessage, TrailingParam, Verb};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Squit {
    server: MiddleParam,
    comment: TrailingParam,
}

impl Squit {
    pub fn new(server: MiddleParam, comment: TrailingParam) -> Squit {
        Squit { server, comment }
    }

    pub fn server(&self) -> &MiddleParam {
        &self.server
    }

    pub fn comment(&self) -> &TrailingParam {
        &self.comment
    }
}

impl ClientMessageParts for Squit {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Squit,
            ParameterList::builder()
                .with_middle(self.server)
                .with_trailing(self.comment),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("SQUIT {} :{}", self.server, self.comment)
    }
}

impl From<Squit> for Message {
    fn from(value: Squit) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Squit> for RawMessage {
    fn from(value: Squit) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Squit {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Squit, ClientMessageError> {
        let (server, comment) = params.try_into()?;
        Ok(Squit { server, comment })
    }
}
