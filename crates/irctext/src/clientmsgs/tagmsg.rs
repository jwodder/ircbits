use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::types::MsgTarget;
use crate::util::{join_with_commas, split_param};
use crate::{Message, MiddleParam, ParameterList, RawMessage, TrailingParam, Verb};

// <https://ircv3.net/specs/extensions/message-tags.html#the-tagmsg-tag-only-message>
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TagMsg {
    targets: Vec<MsgTarget>,
}

impl TagMsg {
    pub fn new<T: Into<MsgTarget>>(target: T) -> TagMsg {
        TagMsg {
            targets: vec![target.into()],
        }
    }

    pub fn new_to_many<I, T>(targets: I) -> Option<TagMsg>
    where
        I: IntoIterator<Item = T>,
        T: Into<MsgTarget>,
    {
        let targets = targets.into_iter().map(Into::into).collect::<Vec<_>>();
        if targets.is_empty() {
            None
        } else {
            Some(TagMsg { targets })
        }
    }

    pub fn targets(&self) -> &[MsgTarget] {
        &self.targets
    }

    pub fn into_targets(self) -> Vec<MsgTarget> {
        self.targets
    }

    fn targets_param(&self) -> MiddleParam {
        assert!(
            !self.targets.is_empty(),
            "TagMsg.targets should always be nonempty"
        );
        let s = join_with_commas(&self.targets).to_string();
        MiddleParam::try_from(s)
            .expect("comma-separated channels and/or nicknames should be a valid MiddleParam")
    }
}

impl ClientMessageParts for TagMsg {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::TagMsg,
            ParameterList::builder()
                .with_middle(self.targets_param())
                .finish(),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("TAGMSG {}", self.targets_param())
    }
}

impl From<TagMsg> for Message {
    fn from(value: TagMsg) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<TagMsg> for RawMessage {
    fn from(value: TagMsg) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for TagMsg {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<TagMsg, ClientMessageError> {
        let (p1,): (TrailingParam,) = params.try_into()?;
        let targets = split_param::<MsgTarget>(p1.as_str())?;
        assert!(
            !targets.is_empty(),
            "targets parsed from TAGMSG message should not be empty"
        );
        Ok(TagMsg { targets })
    }
}
