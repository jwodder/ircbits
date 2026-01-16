use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::DisplayMaybeTrailing;
use crate::{Message, MiddleParam, ParameterList, RawMessage, TrailingParam, Verb};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Stats {
    query: MiddleParam,
    server: Option<TrailingParam>,
}

impl Stats {
    pub fn new(query: MiddleParam) -> Stats {
        Stats {
            query,
            server: None,
        }
    }

    pub fn new_with_server(query: MiddleParam, server: TrailingParam) -> Stats {
        Stats {
            query,
            server: Some(server),
        }
    }

    pub fn query(&self) -> &MiddleParam {
        &self.query
    }

    pub fn server(&self) -> Option<&TrailingParam> {
        self.server.as_ref()
    }
}

impl ClientMessageParts for Stats {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Stats,
            ParameterList::builder()
                .with_middle(self.query)
                .maybe_with_trailing(self.server),
        )
    }

    fn to_irc_line(&self) -> String {
        format!(
            "STATS {}{}",
            self.query,
            DisplayMaybeTrailing(self.server.as_ref())
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
