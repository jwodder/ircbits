use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::types::Channel;
use crate::util::{join_with_commas, split_param};
use crate::{Message, ParameterList, RawMessage, TrailingParam, Verb};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Names {
    channels: Vec<Channel>,
}

impl Names {
    pub fn new(channel: Channel) -> Names {
        Names {
            channels: vec![channel],
        }
    }

    pub fn new_many<I: IntoIterator<Item = Channel>>(channels: I) -> Option<Names> {
        let channels = channels.into_iter().collect::<Vec<_>>();
        if channels.is_empty() {
            None
        } else {
            Some(Names { channels })
        }
    }

    pub fn channels(&self) -> &[Channel] {
        &self.channels
    }

    pub fn into_channels(self) -> Vec<Channel> {
        self.channels
    }

    fn channels_param(&self) -> TrailingParam {
        let s = join_with_commas(&self.channels).to_string();
        TrailingParam::try_from(s).expect("comma-separated channels should be a valid MiddleParam")
    }
}

impl ClientMessageParts for Names {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Names,
            ParameterList::builder().with_trailing(self.channels_param()),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("NAMES :{}", self.channels_param())
    }
}

impl From<Names> for Message {
    fn from(value: Names) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Names> for RawMessage {
    fn from(value: Names) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Names {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Names, ClientMessageError> {
        let (p,): (TrailingParam,) = params.try_into()?;
        let channels = split_param::<Channel>(p.as_str())?;
        // TODO: Error if `channels` is empty?
        Ok(Names { channels })
    }
}
