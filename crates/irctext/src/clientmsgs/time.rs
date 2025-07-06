use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Time {
    server: Option<FinalParam>,
}

impl Time {
    pub fn new() -> Time {
        Time { server: None }
    }

    pub fn new_with_server(server: FinalParam) -> Time {
        Time {
            server: Some(server),
        }
    }

    pub fn server(&self) -> Option<&FinalParam> {
        self.server.as_ref()
    }

    pub fn into_server(self) -> Option<FinalParam> {
        self.server
    }
}

impl ClientMessageParts for Time {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Time,
            ParameterList::builder().maybe_with_final(self.server),
        )
    }
}

impl ToIrcLine for Time {
    fn to_irc_line(&self) -> String {
        let mut s = String::from("TIME");
        if let Some(ref server) = self.server {
            s.push(' ');
            s.push(':');
            s.push_str(server.as_str());
        }
        s
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
