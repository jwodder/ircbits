use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::types::Channel;
use crate::util::DisplayMaybeTrailing;
use crate::{Message, ParameterList, RawMessage, TrailingParam, Verb};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Topic {
    channel: Channel,
    topic: Option<TrailingParam>,
}

impl Topic {
    pub fn new_get(channel: Channel) -> Topic {
        Topic {
            channel,
            topic: None,
        }
    }

    pub fn new_set(channel: Channel, topic: TrailingParam) -> Topic {
        Topic {
            channel,
            topic: Some(topic),
        }
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn topic(&self) -> Option<&TrailingParam> {
        self.topic.as_ref()
    }
}

impl ClientMessageParts for Topic {
    fn into_parts(self) -> (Verb, ParameterList) {
        let params = ParameterList::builder()
            .with_middle(self.channel)
            .maybe_with_trailing(self.topic);
        (Verb::Topic, params)
    }

    fn to_irc_line(&self) -> String {
        format!(
            "TOPIC {}{}",
            self.channel,
            DisplayMaybeTrailing(self.topic.as_ref())
        )
    }
}

impl From<Topic> for Message {
    fn from(value: Topic) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Topic> for RawMessage {
    fn from(value: Topic) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Topic {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Topic, ClientMessageError> {
        let (p1, topic): (_, Option<TrailingParam>) = params.try_into()?;
        let channel = Channel::try_from(p1.into_inner())?;
        Ok(Topic { channel, topic })
    }
}
