use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::DisplayMaybeFinal;
use crate::{FinalParam, Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Admin {
    target: Option<FinalParam>,
}

impl Admin {
    pub fn new() -> Admin {
        Admin { target: None }
    }

    pub fn new_with_target(target: FinalParam) -> Admin {
        Admin {
            target: Some(target),
        }
    }

    pub fn target(&self) -> Option<&FinalParam> {
        self.target.as_ref()
    }

    pub fn into_target(self) -> Option<FinalParam> {
        self.target
    }
}

impl ClientMessageParts for Admin {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Admin,
            ParameterList::builder().maybe_with_final(self.target),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("ADMIN{}", DisplayMaybeFinal(self.target.as_ref()))
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
