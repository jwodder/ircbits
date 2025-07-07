use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::channel::channel_prefixed;
use crate::util::{join_with_commas, split_param};
use crate::{
    Channel, EListCond, MedialParam, Message, ParameterList, ParameterListSizeError, RawMessage,
    Verb,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct List {
    channels: Vec<Channel>,
    elistconds: Vec<EListCond>,
}

impl List {
    pub fn new() -> List {
        List {
            channels: Vec::new(),
            elistconds: Vec::new(),
        }
    }

    pub fn new_with_channels<I: IntoIterator<Item = Channel>>(channels: I) -> List {
        List {
            channels: Vec::from_iter(channels),
            elistconds: Vec::new(),
        }
    }

    pub fn new_with_elistconds<I: IntoIterator<Item = EListCond>>(elistconds: I) -> List {
        List {
            channels: Vec::new(),
            elistconds: Vec::from_iter(elistconds),
        }
    }

    pub fn new_with_channels_and_elistconds<I, J>(channels: I, elistconds: J) -> List
    where
        I: IntoIterator<Item = Channel>,
        J: IntoIterator<Item = EListCond>,
    {
        List {
            channels: Vec::from_iter(channels),
            elistconds: Vec::from_iter(elistconds),
        }
    }

    pub fn channels(&self) -> &[Channel] {
        &self.channels
    }

    pub fn elistconds(&self) -> &[EListCond] {
        &self.elistconds
    }

    fn channels_param(&self) -> Option<MedialParam> {
        if self.channels.is_empty() {
            None
        } else {
            let s = join_with_commas(&self.channels);
            Some(
                MedialParam::try_from(s)
                    .expect("comma-separated channels should be a valid MedialParam"),
            )
        }
    }

    fn elistconds_param(&self) -> Option<MedialParam> {
        if self.elistconds.is_empty() {
            None
        } else {
            let s = join_with_commas(&self.elistconds);
            Some(
                MedialParam::try_from(s)
                    .expect("comma-separated elistconds should be a valid MedialParam"),
            )
        }
    }
}

impl ClientMessageParts for List {
    fn into_parts(self) -> (Verb, ParameterList) {
        let mut builder = ParameterList::builder();
        if let Some(channels) = self.channels_param() {
            builder.push_medial(channels);
        }
        if let Some(elistconds) = self.elistconds_param() {
            builder.push_medial(elistconds);
        }
        (Verb::List, builder.finish())
    }

    fn to_irc_line(&self) -> String {
        let mut s = String::from("LIST");
        if let Some(channels) = self.channels_param() {
            s.push(' ');
            s.push_str(channels.as_str());
        }
        if let Some(elistconds) = self.elistconds_param() {
            s.push(' ');
            s.push_str(elistconds.as_str());
        }
        s
    }
}

impl From<List> for Message {
    fn from(value: List) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<List> for RawMessage {
    fn from(value: List) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for List {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<List, ClientMessageError> {
        let len = params.len();
        if (0..=2).contains(&len) {
            let mut iter = params.into_iter();
            let channels;
            let elistconds;
            if let Some(p) = iter.next() {
                if channel_prefixed(p.as_str()) {
                    channels = split_param::<Channel>(p.as_str())?;
                    if let Some(p2) = iter.next() {
                        elistconds = split_param::<EListCond>(p2.as_str())?;
                    } else {
                        elistconds = Vec::new();
                    }
                } else {
                    channels = Vec::new();
                    elistconds = split_param::<EListCond>(p.as_str())?;
                    if iter.next().is_some() {
                        return Err(ClientMessageError::ParamQty(
                            ParameterListSizeError::Exact {
                                requested: 1,
                                received: len,
                            },
                        ));
                    }
                }
            } else {
                channels = Vec::new();
                elistconds = Vec::new();
            }
            Ok(List {
                channels,
                elistconds,
            })
        } else {
            Err(ClientMessageError::ParamQty(
                ParameterListSizeError::Range {
                    min_requested: 0,
                    max_requested: 2,
                    received: len,
                },
            ))
        }
    }
}
