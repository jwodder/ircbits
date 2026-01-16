use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::types::Channel;
use crate::util::{DisplayMaybeTrailing, join_with_commas, split_param};
use crate::{Message, MiddleParam, ParameterList, RawMessage, TrailingParam, Verb};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Part {
    channels: Vec<Channel>,
    reason: Option<TrailingParam>,
}

impl Part {
    pub fn new(channel: Channel) -> Part {
        Part {
            channels: vec![channel],
            reason: None,
        }
    }

    pub fn new_with_reason(channel: Channel, reason: TrailingParam) -> Part {
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
        reason: TrailingParam,
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

    pub fn reason(&self) -> Option<&TrailingParam> {
        self.reason.as_ref()
    }

    fn channels_param(&self) -> MiddleParam {
        assert!(
            !self.channels.is_empty(),
            "Part.channels should always be nonempty"
        );
        let s = join_with_commas(&self.channels).to_string();
        MiddleParam::try_from(s).expect("comma-separated channels should be a valid MiddleParam")
    }
}

impl ClientMessageParts for Part {
    fn into_parts(self) -> (Verb, ParameterList) {
        let params = ParameterList::builder()
            .with_middle(self.channels_param())
            .maybe_with_trailing(self.reason);
        (Verb::Part, params)
    }

    fn to_irc_line(&self) -> String {
        format!(
            "PART {}{}",
            self.channels_param(),
            DisplayMaybeTrailing(self.reason.as_ref())
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
        let (p1, reason): (_, Option<TrailingParam>) = params.try_into()?;
        let channels = split_param::<Channel>(p1.as_str())?;
        assert!(
            !channels.is_empty(),
            "channels parsed from PART message should not be empty"
        );
        Ok(Part { channels, reason })
    }
}
