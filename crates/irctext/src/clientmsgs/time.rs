use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Time(Option<FinalParam>);

impl Time {
    pub fn new() -> Time {
        Time(None)
    }

    pub fn new_with_server(server: FinalParam) -> Time {
        Time(Some(server))
    }

    pub fn server(&self) -> Option<&FinalParam> {
        self.0.as_ref()
    }

    pub fn into_server(self) -> Option<FinalParam> {
        self.0
    }
}

impl ClientMessageParts for Time {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Time,
            ParameterList::builder().maybe_with_final(self.0),
        )
    }
}

impl ToIrcLine for Time {
    fn to_irc_line(&self) -> String {
        let mut s = String::from("TIME");
        if let Some(ref server) = self.0 {
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
        if params.is_empty() {
            Ok(Time::new())
        } else {
            let (p,) = params.try_into()?;
            Ok(Time::new_with_server(p))
        }
    }
}
