use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::DisplayMaybeFinal;
use crate::{Channel, FinalParam, Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Topic {
    channel: Channel,
    topic: Option<FinalParam>,
}

impl Topic {
    pub fn new_get(channel: Channel) -> Topic {
        Topic {
            channel,
            topic: None,
        }
    }

    pub fn new_set(channel: Channel, topic: FinalParam) -> Topic {
        Topic {
            channel,
            topic: Some(topic),
        }
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn topic(&self) -> Option<&FinalParam> {
        self.topic.as_ref()
    }
}

impl ClientMessageParts for Topic {
    fn into_parts(self) -> (Verb, ParameterList) {
        let params = ParameterList::builder()
            .with_medial(self.channel)
            .maybe_with_final(self.topic);
        (Verb::Topic, params)
    }

    fn to_irc_line(&self) -> String {
        format!(
            "TOPIC {}{}",
            self.channel,
            DisplayMaybeFinal(self.topic.as_ref())
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
        let (p1, topic): (_, Option<FinalParam>) = params.try_into()?;
        match p1.as_str().parse::<Channel>() {
            Ok(channel) => Ok(Topic { channel, topic }),
            Err(source) => Err(ClientMessageError::ParseParam {
                index: 0,
                raw: p1.into_inner(),
                source: Box::new(source),
            }),
        }
    }
}
