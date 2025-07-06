use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Channel, FinalParam, Message, ParameterList, RawMessage, ToIrcLine, Verb};

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
}

impl ToIrcLine for Topic {
    fn to_irc_line(&self) -> String {
        let mut s = format!("TOPIC {}", self.channel);
        if let Some(ref topic) = self.topic {
            s.push(' ');
            s.push(':');
            s.push_str(topic.as_str());
        }
        s
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
        let (p1, p2): (_, Option<FinalParam>) = params.try_into()?;
        match p1.as_str().parse::<Channel>() {
            Ok(channel) => Ok(Topic { channel, topic: p2 }),
            Err(source) => Err(ClientMessageError::ParseParam {
                index: 0,
                raw: p1.into_inner(),
                source: Box::new(source),
            }),
        }
    }
}
