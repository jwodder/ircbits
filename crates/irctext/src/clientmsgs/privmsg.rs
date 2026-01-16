use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::types::MsgTarget;
use crate::util::{join_with_commas, split_param};
use crate::{Message, MiddleParam, ParameterList, RawMessage, TrailingParam, Verb};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PrivMsg {
    targets: Vec<MsgTarget>,
    text: TrailingParam,
}

impl PrivMsg {
    pub fn new<T: Into<MsgTarget>>(target: T, text: TrailingParam) -> PrivMsg {
        PrivMsg {
            targets: vec![target.into()],
            text,
        }
    }

    pub fn new_to_many<I, T>(targets: I, text: TrailingParam) -> Option<PrivMsg>
    where
        I: IntoIterator<Item = T>,
        T: Into<MsgTarget>,
    {
        let targets = targets.into_iter().map(Into::into).collect::<Vec<_>>();
        if targets.is_empty() {
            None
        } else {
            Some(PrivMsg { targets, text })
        }
    }

    pub fn targets(&self) -> &[MsgTarget] {
        &self.targets
    }

    pub fn text(&self) -> &TrailingParam {
        &self.text
    }

    fn targets_param(&self) -> MiddleParam {
        assert!(
            !self.targets.is_empty(),
            "PrivMsg.targets should always be nonempty"
        );
        let s = join_with_commas(&self.targets).to_string();
        MiddleParam::try_from(s)
            .expect("comma-separated channels and/or nicknames should be a valid MiddleParam")
    }
}

impl ClientMessageParts for PrivMsg {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::PrivMsg,
            ParameterList::builder()
                .with_middle(self.targets_param())
                .with_trailing(self.text),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("PRIVMSG {} :{}", self.targets_param(), self.text)
    }
}

impl From<PrivMsg> for Message {
    fn from(value: PrivMsg) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<PrivMsg> for RawMessage {
    fn from(value: PrivMsg) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for PrivMsg {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<PrivMsg, ClientMessageError> {
        let (p1, text): (_, TrailingParam) = params.try_into()?;
        let targets = split_param::<MsgTarget>(p1.as_str())?;
        assert!(
            !targets.is_empty(),
            "targets parsed from PRIVMSG message should not be empty"
        );
        Ok(PrivMsg { targets, text })
    }
}
