use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::types::Username;
use crate::{Message, MiddleParam, ParameterList, RawMessage, TrailingParam, Verb};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct User {
    username: Username,
    realname: TrailingParam,
}

impl User {
    pub fn new(username: Username, realname: TrailingParam) -> User {
        User { username, realname }
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn realname(&self) -> &TrailingParam {
        &self.realname
    }
}

impl ClientMessageParts for User {
    fn into_parts(self) -> (Verb, ParameterList) {
        let params = ParameterList::builder()
            .with_middle(self.username)
            .with_middle(
                "0".parse::<MiddleParam>()
                    .expect(r#""0" should be a valid MiddleParam"#),
            )
            .with_middle(
                "*".parse::<MiddleParam>()
                    .expect(r#""*" should be a valid MiddleParam"#),
            )
            .with_trailing(self.realname);
        (Verb::User, params)
    }

    fn to_irc_line(&self) -> String {
        format!("USER {} 0 * :{}", self.username, self.realname)
    }
}

impl From<User> for Message {
    fn from(value: User) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<User> for RawMessage {
    fn from(value: User) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for User {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<User, ClientMessageError> {
        let (username, _, _, realname) = params.try_into()?;
        let username = Username::try_from(username.into_inner())?;
        Ok(User { username, realname })
    }
}
