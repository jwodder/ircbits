use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::DisplayMaybeTrailing;
use crate::{Message, ParameterList, RawMessage, TrailingParam, Verb};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Admin {
    target: Option<TrailingParam>,
}

impl Admin {
    pub fn new() -> Admin {
        Admin { target: None }
    }

    pub fn new_with_target(target: TrailingParam) -> Admin {
        Admin {
            target: Some(target),
        }
    }

    pub fn target(&self) -> Option<&TrailingParam> {
        self.target.as_ref()
    }

    pub fn into_target(self) -> Option<TrailingParam> {
        self.target
    }
}

impl ClientMessageParts for Admin {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Admin,
            ParameterList::builder().maybe_with_trailing(self.target),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("ADMIN{}", DisplayMaybeTrailing(self.target.as_ref()))
    }
}

impl From<Admin> for Message {
    fn from(value: Admin) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Admin> for RawMessage {
    fn from(value: Admin) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Admin {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Admin, ClientMessageError> {
        let (target,) = params.try_into()?;
        Ok(Admin { target })
    }
}
