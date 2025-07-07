use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::{join_with_commas, split_param};
use crate::{FinalParam, MedialParam, Message, ParameterList, RawMessage, Target, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivMsg {
    targets: Vec<Target>,
    text: FinalParam,
}

impl PrivMsg {
    pub fn new<T: Into<Target>>(target: T, text: FinalParam) -> PrivMsg {
        PrivMsg {
            targets: vec![target.into()],
            text,
        }
    }

    pub fn new_to_many<I, T>(targets: I, text: FinalParam) -> Option<PrivMsg>
    where
        I: IntoIterator<Item = T>,
        T: Into<Target>,
    {
        let targets = targets.into_iter().map(Into::into).collect::<Vec<_>>();
        if targets.is_empty() {
            None
        } else {
            Some(PrivMsg { targets, text })
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
            "PrivMsg.targets should always be nonempty"
        );
        let s = join_with_commas(&self.targets);
        MedialParam::try_from(s)
            .expect("comma-separated channels and/or nicknames should be a valid MedialParam")
    }
}

impl ClientMessageParts for PrivMsg {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::PrivMsg,
            ParameterList::builder()
                .with_medial(self.targets_param())
                .with_final(self.text),
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
        let (p1, text): (_, FinalParam) = params.try_into()?;
        let targets = split_param::<Target>(p1.as_str())?;
        Ok(PrivMsg { targets, text })
    }
}
