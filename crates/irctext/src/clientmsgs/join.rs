use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::types::{Channel, Key};
use crate::util::{DisplayMaybeFinal, join_with_commas, split_param};
use crate::{
    FinalParam, MedialParam, Message, ParameterList, ParameterListSizeError, RawMessage, Verb,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Join(InnerJoin);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum InnerJoin {
    Channels {
        channels: Vec<Channel>,
        keys: Vec<Key>,
    },
    Zero,
}

impl Join {
    pub fn new(channel: Channel) -> Join {
        Join(InnerJoin::Channels {
            channels: vec![channel],
            keys: Vec::new(),
        })
    }

    pub fn new_with_key(channel: Channel, key: Key) -> Join {
        Join(InnerJoin::Channels {
            channels: vec![channel],
            keys: vec![key],
        })
    }

    pub fn new_multi<I: IntoIterator<Item = Channel>>(channels: I) -> Option<Join> {
        let channels = channels.into_iter().collect::<Vec<_>>();
        if channels.is_empty() {
            None
        } else {
            Some(Join(InnerJoin::Channels {
                channels,
                keys: Vec::new(),
            }))
        }
    }

    pub fn new_multi_with_keys<I: IntoIterator<Item = (Channel, Key)>>(iter: I) -> Option<Join> {
        let mut channels = Vec::new();
        let mut keys = Vec::new();
        for (chan, k) in iter {
            channels.push(chan);
            keys.push(k);
        }
        if channels.is_empty() {
            None
        } else {
            Some(Join(InnerJoin::Channels { channels, keys }))
        }
    }

    pub fn new_zero() -> Join {
        Join(InnerJoin::Zero)
    }

    pub fn is_zero(&self) -> bool {
        matches!(self.0, InnerJoin::Zero)
    }

    pub fn channels(&self) -> &[Channel] {
        match &self.0 {
            InnerJoin::Channels { channels, .. } => channels,
            InnerJoin::Zero => &[],
        }
    }

    pub fn keys(&self) -> &[Key] {
        match &self.0 {
            InnerJoin::Channels { keys, .. } => keys,
            InnerJoin::Zero => &[],
        }
    }

    fn channels_param(&self) -> MedialParam {
        if self.is_zero() {
            "0".parse::<MedialParam>()
                .expect(r#""0" should be a valid MedialParam"#)
        } else {
            let channels = self.channels();
            assert!(
                !channels.is_empty(),
                "Join.channels should always be nonempty"
            );
            let s = join_with_commas(channels).to_string();
            MedialParam::try_from(s)
                .expect("comma-separated channels should be a valid MedialParam")
        }
    }

    fn keys_param(&self) -> Option<FinalParam> {
        let keys = self.keys();
        if keys.is_empty() {
            None
        } else {
            let s = join_with_commas(keys).to_string();
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
        if p1 == "0" {
            if p2.is_some() {
                return Err(ClientMessageError::ParamQty(
                    ParameterListSizeError::Exact {
                        required: 1,
                        received: 2,
                    },
                ));
            }
            Ok(Join(InnerJoin::Zero))
        } else {
            let channels = split_param::<Channel>(p1.as_str())?;
            assert!(
                !channels.is_empty(),
                "channels parsed from JOIN message should not be empty"
            );
            let keys = match p2 {
                Some(p) => split_param::<Key>(p.as_str())?,
                None => Vec::new(),
            };
            Ok(Join(InnerJoin::Channels { channels, keys }))
        }
    }
}
