use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::DisplayMaybeFinal;
use crate::{FinalParam, MedialParam, Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stats {
    query: MedialParam,
    server: Option<FinalParam>,
}

impl Stats {
    pub fn new(query: MedialParam) -> Stats {
        Stats {
            query,
            server: None,
        }
    }

    pub fn new_with_server(query: MedialParam, server: FinalParam) -> Stats {
        Stats {
            query,
            server: Some(server),
        }
    }

    pub fn query(&self) -> &MedialParam {
        &self.query
    }

    pub fn server(&self) -> Option<&FinalParam> {
        self.server.as_ref()
    }
}

impl ClientMessageParts for Stats {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Stats,
            ParameterList::builder()
                .with_medial(self.query)
                .maybe_with_final(self.server),
        )
    }

    fn to_irc_line(&self) -> String {
        format!(
            "STATS {}{}",
            self.query,
            DisplayMaybeFinal(self.server.as_ref())
        )
    }
}

impl From<Stats> for Message {
    fn from(value: Stats) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Stats> for RawMessage {
    fn from(value: Stats) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Stats {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Stats, ClientMessageError> {
        let (query, server) = params.try_into()?;
        Ok(Stats { query, server })
    }
}
