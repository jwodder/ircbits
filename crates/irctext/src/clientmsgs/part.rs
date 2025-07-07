use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::{join_with_commas, split_param, DisplayMaybeFinal};
use crate::{Channel, FinalParam, MedialParam, Message, ParameterList, RawMessage, Verb};

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
            "Part.channels should always be nonempty"
        );
        let s = join_with_commas(&self.channels);
        MedialParam::try_from(s).expect("comma-separated channels should be a valid MedialParam")
    }
}

impl ClientMessageParts for Part {
    fn into_parts(self) -> (Verb, ParameterList) {
        let params = ParameterList::builder()
            .with_medial(self.channels_param())
            .maybe_with_final(self.reason);
        (Verb::Part, params)
    }

    fn to_irc_line(&self) -> String {
        format!(
            "PART {}{}",
            self.channels_param(),
            DisplayMaybeFinal(self.reason.as_ref())
        )
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
        let (p1, reason): (_, Option<FinalParam>) = params.try_into()?;
        let channels = split_param::<Channel>(p1.as_str())?;
        assert!(
            !channels.is_empty(),
            "channels parsed from PART message should not be empty"
        );
        Ok(Part { channels, reason })
    }
}
