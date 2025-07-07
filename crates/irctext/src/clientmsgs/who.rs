use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, MedialParam, Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Who {
    mask: MedialParam,
}

impl Who {
    pub fn new<P: Into<MedialParam>>(mask: P) -> Who {
        Who { mask: mask.into() }
    }

    pub fn mask(&self) -> &MedialParam {
        &self.mask
    }

    pub fn into_mask(self) -> MedialParam {
        self.mask
    }
}

impl ClientMessageParts for Who {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Who,
            ParameterList::builder().with_medial(self.mask).finish(),
        )
    }
}

impl ToIrcLine for Who {
    fn to_irc_line(&self) -> String {
        format!("WHO {}", self.mask)
    }
}

impl From<Who> for Message {
    fn from(value: Who) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Who> for RawMessage {
    fn from(value: Who) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Who {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Who, ClientMessageError> {
        let (p,): (FinalParam,) = params.try_into()?;
        match p.as_str().parse::<MedialParam>() {
            Ok(mask) => Ok(Who { mask }),
            Err(source) => Err(ClientMessageError::ParseParam {
                index: 0,
                raw: p.into_inner(),
                source: Box::new(source),
            }),
        }
    }
}
