use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::DisplayMaybeTrailing;
use crate::{Message, ParameterList, RawMessage, TrailingParam, Verb};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Time {
    server: Option<TrailingParam>,
}

impl Time {
    pub fn new() -> Time {
        Time { server: None }
    }

    pub fn new_with_server(server: TrailingParam) -> Time {
        Time {
            server: Some(server),
        }
    }

    pub fn server(&self) -> Option<&TrailingParam> {
        self.server.as_ref()
    }

    pub fn into_server(self) -> Option<TrailingParam> {
        self.server
    }
}

impl ClientMessageParts for Time {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Time,
            ParameterList::builder().maybe_with_trailing(self.server),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("TIME{}", DisplayMaybeTrailing(self.server.as_ref()))
    }
}

impl From<Time> for Message {
    fn from(value: Time) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Time> for RawMessage {
    fn from(value: Time) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Time {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Time, ClientMessageError> {
        let (server,) = params.try_into()?;
        Ok(Time { server })
    }
}
