use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::types::MsgTarget;
use crate::util::{join_with_commas, split_param};
use crate::{FinalParam, MedialParam, Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Notice {
    targets: Vec<MsgTarget>,
    text: FinalParam,
}

impl Notice {
    pub fn new<T: Into<MsgTarget>>(target: T, text: FinalParam) -> Notice {
        Notice {
            targets: vec![target.into()],
            text,
        }
    }

    pub fn new_to_many<I, T>(targets: I, text: FinalParam) -> Option<Notice>
    where
        I: IntoIterator<Item = T>,
        T: Into<MsgTarget>,
    {
        let targets = targets.into_iter().map(Into::into).collect::<Vec<_>>();
        if targets.is_empty() {
            None
        } else {
            Some(Notice { targets, text })
        }
    }

    pub fn targets(&self) -> &[MsgTarget] {
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
        let s = join_with_commas(&self.targets).to_string();
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
        let targets = split_param::<MsgTarget>(p1.as_str())?;
        assert!(
            !targets.is_empty(),
            "targets parsed from NOTICE message should not be empty"
        );
        Ok(Notice { targets, text })
    }
}
