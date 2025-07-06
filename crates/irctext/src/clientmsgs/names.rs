use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::{join_with_commas, split_channels};
use crate::{Channel, MedialParam, Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Names(Vec<Channel>);

impl Names {
    pub fn new(channel: Channel) -> Names {
        Names(vec![channel])
    }

    pub fn new_many<I: IntoIterator<Item = Channel>>(channels: I) -> Option<Names> {
        let channels = channels.into_iter().collect::<Vec<_>>();
        if channels.is_empty() {
            None
        } else {
            Some(Names(channels))
        }
    }

    pub fn channels(&self) -> &[Channel] {
        &self.0
    }

    fn channels_param(&self) -> MedialParam {
        assert!(!self.0.is_empty(), "Names.0 should always be nonempty");
        let s = join_with_commas(&self.0);
        MedialParam::try_from(s).expect("comma-separated channels should be a valid MedialParam")
    }
}

impl ClientMessageParts for Names {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Names,
            ParameterList::builder()
                .with_medial(self.channels_param())
                .finish(),
        )
    }
}

impl ToIrcLine for Names {
    fn to_irc_line(&self) -> String {
        format!("NAMES {}", self.channels_param())
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
        let (p,) = params.try_into()?;
        let channels = split_channels(p.into_inner())?;
        Ok(Names(channels))
    }
}
