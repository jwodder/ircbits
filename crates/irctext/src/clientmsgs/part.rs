use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::{join_with_commas, split_channels};
use crate::{
    Channel, FinalParam, MedialParam, Message, ParameterList, RawMessage, ToIrcLine, Verb,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Part {
    channels: Vec<Channel>,
    reason: Option<FinalParam>,
}

impl Part {
    pub fn new(channel: Channel) -> Part {
        Part {
            channels: vec![channel],
            reason: None,
        }
    }

    pub fn new_with_reason(channel: Channel, reason: FinalParam) -> Part {
        Part {
            channels: vec![channel],
            reason: Some(reason),
        }
    }

    pub fn new_many<I: IntoIterator<Item = Channel>>(channels: I) -> Option<Part> {
        let channels = channels.into_iter().collect::<Vec<_>>();
        if channels.is_empty() {
            None
        } else {
            Some(Part {
                channels,
                reason: None,
            })
        }
    }

    pub fn new_many_with_reason<I: IntoIterator<Item = Channel>>(
        channels: I,
        reason: FinalParam,
    ) -> Option<Part> {
        let channels = channels.into_iter().collect::<Vec<_>>();
        if channels.is_empty() {
            None
        } else {
            Some(Part {
                channels,
                reason: Some(reason),
            })
        }
    }

    pub fn channels(&self) -> &[Channel] {
        &self.channels
    }

    pub fn reason(&self) -> Option<&FinalParam> {
        self.reason.as_ref()
    }

    fn channels_param(&self) -> MedialParam {
        assert!(
            !self.channels.is_empty(),
            "Join.channels should always be nonempty"
        );
        let s = join_with_commas(&self.channels);
        MedialParam::try_from(s).expect("comma-separated channels should be a valid MedialParam")
    }
}

impl ClientMessageParts for Part {
    fn into_parts(self) -> (Verb, ParameterList) {
        let builder = ParameterList::builder().with_medial(self.channels_param());
        let params = if let Some(reason) = self.reason {
            builder.with_final(reason)
        } else {
            builder.finish()
        };
        (Verb::Part, params)
    }
}

impl ToIrcLine for Part {
    fn to_irc_line(&self) -> String {
        let mut s = format!("PART {}", self.channels_param());
        if let Some(ref reason) = self.reason {
            s.push(' ');
            s.push(':');
            s.push_str(reason.as_str());
        }
        s
    }
}

impl From<Part> for Message {
    fn from(value: Part) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Part> for RawMessage {
    fn from(value: Part) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Part {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Part, ClientMessageError> {
        if params.len() == 1 {
            let (p,) = params.try_into()?;
            let channels = split_channels(p.into_inner())?;
            Ok(Part {
                channels,
                reason: None,
            })
        } else {
            let (p1, p2) = params.try_into()?;
            let channels = split_channels(p1.into_inner())?;
            Ok(Part {
                channels,
                reason: Some(p2),
            })
        }
    }
}
