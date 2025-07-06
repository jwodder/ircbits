use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, Message, ParameterList, RawMessage, ToIrcLine, Verb};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Admin(Option<FinalParam>);

impl Admin {
    pub fn new() -> Admin {
        Admin(None)
    }

    pub fn new_with_target(target: FinalParam) -> Admin {
        Admin(Some(target))
    }

    pub fn target(&self) -> Option<&FinalParam> {
        self.0.as_ref()
    }

    pub fn into_target(self) -> Option<FinalParam> {
        self.0
    }
}

impl ClientMessageParts for Admin {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Admin,
            ParameterList::builder().maybe_with_final(self.0),
        )
    }
}

impl ToIrcLine for Admin {
    fn to_irc_line(&self) -> String {
        let mut s = String::from("ADMIN");
        if let Some(ref target) = self.0 {
            s.push(' ');
            s.push(':');
            s.push_str(target.as_str());
        }
        s
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
        if params.is_empty() {
            Ok(Admin::new())
        } else {
            let (p,) = params.try_into()?;
            Ok(Admin::new_with_target(p))
        }
    }
}
