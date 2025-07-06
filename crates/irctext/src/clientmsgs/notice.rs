use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::{join_with_commas, split_targets};
use crate::{FinalParam, MedialParam, Message, ParameterList, RawMessage, Target, ToIrcLine, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Notice {
    targets: Vec<Target>,
    text: FinalParam,
}

impl Notice {
    pub fn new<T: Into<Target>>(target: T, text: FinalParam) -> Notice {
        Notice {
            targets: vec![target.into()],
            text,
        }
    }

    pub fn new_to_many<I, T>(targets: I, text: FinalParam) -> Option<Notice>
    where
        I: IntoIterator<Item = T>,
        T: Into<Target>,
    {
        let targets = targets.into_iter().map(Into::into).collect::<Vec<_>>();
        if targets.is_empty() {
            None
        } else {
            Some(Notice { targets, text })
        }
    }

    pub fn targets(&self) -> &[Target] {
        &self.targets
    }

    pub fn text(&self) -> &FinalParam {
        &self.text
    }

    fn targets_param(&self) -> MedialParam {
        assert!(
            !self.targets.is_empty(),
            "Notice.targets should always be nonempty"
        );
        let s = join_with_commas(&self.targets);
        MedialParam::try_from(s)
            .expect("comma-separated channels and/or nicknames should be a valid MedialParam")
    }
}

impl ClientMessageParts for Notice {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Notice,
            ParameterList::builder()
                .with_medial(self.targets_param())
                .with_final(self.text),
        )
    }
}

impl ToIrcLine for Notice {
    fn to_irc_line(&self) -> String {
        format!("NOTICE {} :{}", self.targets_param(), self.text)
    }
}

impl From<Notice> for Message {
    fn from(value: Notice) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Notice> for RawMessage {
    fn from(value: Notice) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Notice {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Notice, ClientMessageError> {
        let (p1, text): (_, FinalParam) = params.try_into()?;
        let targets = split_targets(p1.into_inner())?;
        Ok(Notice { targets, text })
    }
}
