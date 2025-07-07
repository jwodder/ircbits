use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::{join_with_commas, split_param, DisplayMaybeFinal};
use crate::{Channel, FinalParam, Key, MedialParam, Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Join {
    channels: Vec<Channel>,
    keys: Vec<Key>,
}

impl Join {
    pub fn new_channel(channel: Channel) -> Join {
        Join {
            channels: vec![channel],
            keys: Vec::new(),
        }
    }

    pub fn new_channel_with_key(channel: Channel, key: Key) -> Join {
        Join {
            channels: vec![channel],
            keys: vec![key],
        }
    }

    pub fn new_channels<I: IntoIterator<Item = Channel>>(channels: I) -> Option<Join> {
        let channels = channels.into_iter().collect::<Vec<_>>();
        if channels.is_empty() {
            None
        } else {
            Some(Join {
                channels,
                keys: Vec::new(),
            })
        }
    }

    pub fn new_channels_with_keys<I: IntoIterator<Item = (Channel, Key)>>(iter: I) -> Option<Join> {
        let mut channels = Vec::new();
        let mut keys = Vec::new();
        for (chan, k) in iter {
            channels.push(chan);
            keys.push(k);
        }
        if channels.is_empty() {
            None
        } else {
            Some(Join { channels, keys })
        }
    }

    pub fn new_zero() -> Join {
        let Ok(zero) = "0".parse::<Channel>() else {
            unreachable!(r#""0" should be a valid Channel"#);
        };
        Join::new_channel(zero)
    }

    pub fn channels(&self) -> &[Channel] {
        &self.channels
    }

    pub fn keys(&self) -> &[Key] {
        &self.keys
    }

    fn channels_param(&self) -> MedialParam {
        assert!(
            !self.channels.is_empty(),
            "Join.channels should always be nonempty"
        );
        let s = join_with_commas(&self.channels);
        MedialParam::try_from(s).expect("comma-separated channels should be a valid MedialParam")
    }

    fn keys_param(&self) -> Option<FinalParam> {
        if self.keys.is_empty() {
            None
        } else {
            let s = join_with_commas(&self.keys);
            Some(
                FinalParam::try_from(s).expect("comma-separated keys should be a valid FinalParam"),
            )
        }
    }
}

impl ClientMessageParts for Join {
    fn into_parts(self) -> (Verb, ParameterList) {
        let params = ParameterList::builder()
            .with_medial(self.channels_param())
            .maybe_with_final(self.keys_param());
        (Verb::Join, params)
    }

    fn to_irc_line(&self) -> String {
        format!(
            "JOIN {}{}",
            self.channels_param(),
            DisplayMaybeFinal(self.keys_param())
        )
    }
}

impl From<Join> for Message {
    fn from(value: Join) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Join> for RawMessage {
    fn from(value: Join) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Join {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Join, ClientMessageError> {
        let (p1, p2): (_, Option<FinalParam>) = params.try_into()?;
        let channels = split_param::<Channel>(p1.as_str())?;
        assert!(
            !channels.is_empty(),
            "channels parsed from JOIN message should not be empty"
        );
        let keys = match p2 {
            Some(p) => split_param::<Key>(p.as_str())?,
            None => Vec::new(),
        };
        Ok(Join { channels, keys })
    }
}
