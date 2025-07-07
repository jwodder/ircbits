use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, MedialParam, Message, ParameterList, RawMessage, Username, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User {
    username: Username,
    realname: FinalParam,
}

impl User {
    pub fn new(username: Username, realname: FinalParam) -> User {
        User { username, realname }
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn realname(&self) -> &FinalParam {
        &self.realname
    }
}

impl ClientMessageParts for User {
    fn into_parts(self) -> (Verb, ParameterList) {
        let params = ParameterList::builder()
            .with_medial(self.username)
            .with_medial(
                "0".parse::<MedialParam>()
                    .expect(r#""0" should be a valid MedialParam"#),
            )
            .with_medial(
                "*".parse::<MedialParam>()
                    .expect(r#""*" should be a valid MedialParam"#),
            )
            .with_final(self.realname);
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
