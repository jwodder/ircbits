use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{
    Channel, FinalParam, Key, MedialParam, Message, ParameterList, RawMessage, ToIrcLine, Verb,
};

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

    pub fn new_channels<I: IntoIterator<Item = Channel>>(iter: I) -> Option<Join> {
        let channels = iter.into_iter().collect::<Vec<_>>();
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
        let mut s = String::new();
        let mut first = true;
        assert!(
            !self.channels.is_empty(),
            "Join.channels should always be nonempty"
        );
        for chan in &self.channels {
            if !std::mem::replace(&mut first, false) {
                s.push(',');
            }
            s.push_str(chan.as_str());
        }
        MedialParam::try_from(s).expect("comma-separated channels should be a valid MedialParam")
    }

    fn keys_param(&self) -> Option<FinalParam> {
        if self.keys.is_empty() {
            None
        } else {
            let mut s = String::new();
            let mut first = true;
            for key in &self.keys {
                if !std::mem::replace(&mut first, false) {
                    s.push(',');
                }
                s.push_str(key.as_str());
            }
            Some(
                FinalParam::try_from(s).expect("comma-separated keys should be a valid FinalParam"),
            )
        }
    }
}

impl ClientMessageParts for Join {
    fn into_parts(self) -> (Verb, ParameterList) {
        let builder = ParameterList::builder().with_medial(self.channels_param());
        let params = if let Some(keys_param) = self.keys_param() {
            builder.with_final(keys_param)
        } else {
            builder.finish()
        };
        (Verb::Join, params)
    }
}

impl ToIrcLine for Join {
    fn to_irc_line(&self) -> String {
        let mut s = format!("JOIN {}", self.channels_param());
        if let Some(keys_param) = self.keys_param() {
            s.push(' ');
            s.push(':');
            s.push_str(keys_param.as_str());
        }
        s
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
        if params.len() == 1 {
            let (p,) = params.try_into()?;
            let channels = split_channels(p.into_inner())?;
            Ok(Join {
                channels,
                keys: Vec::new(),
            })
        } else {
            let (p1, p2) = params.try_into()?;
            let channels = split_channels(p1.into_inner())?;
            let keys = split_keys(p2.into_inner())?;
            Ok(Join { channels, keys })
        }
    }
}

fn split_channels(s: String) -> Result<Vec<Channel>, ClientMessageError> {
    match s
        .split(',')
        .map(str::parse::<Channel>)
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(channels) => Ok(channels),
        Err(source) => Err(ClientMessageError::ParseParam {
            index: 0,
            raw: s,
            source: Box::new(source),
        }),
    }
}

fn split_keys(s: String) -> Result<Vec<Key>, ClientMessageError> {
    match s
        .split(',')
        .map(str::parse::<Key>)
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(keys) => Ok(keys),
        Err(source) => Err(ClientMessageError::ParseParam {
            index: 1,
            raw: s,
            source: Box::new(source),
        }),
    }
}
