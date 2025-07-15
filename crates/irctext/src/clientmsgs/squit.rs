use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, MedialParam, Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Squit {
    server: MedialParam,
    comment: FinalParam,
}

impl Squit {
    pub fn new(server: MedialParam, comment: FinalParam) -> Squit {
        Squit { server, comment }
    }

    pub fn server(&self) -> &MedialParam {
        &self.server
    }

    pub fn comment(&self) -> &FinalParam {
        &self.comment
    }
}

impl ClientMessageParts for Squit {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Squit,
            ParameterList::builder()
                .with_medial(self.server)
                .with_final(self.comment),
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
