use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, Nickname, ParameterList, ParameterListSizeError, RawMessage, Verb};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserHost {
    nicknames: Vec<Nickname>,
}

impl UserHost {
    pub fn new<I: IntoIterator<Item = Nickname>>(nicknames: I) -> Result<UserHost, UserHostError> {
        let nicknames = Vec::from_iter(nicknames);
        if (1..=5).contains(&nicknames.len()) {
            Ok(UserHost { nicknames })
        } else {
            Err(UserHostError(nicknames.len()))
        }
    }

    pub fn nicknames(&self) -> &[Nickname] {
        &self.nicknames
    }

    pub fn into_nicknames(self) -> Vec<Nickname> {
        self.nicknames
    }
}

impl ClientMessageParts for UserHost {
    fn into_parts(self) -> (Verb, ParameterList) {
        let mut builder = ParameterList::builder();
        for nick in self.nicknames {
            builder.push_medial(nick);
        }
        (Verb::UserHost, builder.finish())
    }

    fn to_irc_line(&self) -> String {
        let mut s = String::from("USERHOST");
        for nick in self.nicknames() {
            s.push(' ');
            s.push_str(nick.as_str());
        }
        s
    }
}

impl From<UserHost> for Message {
    fn from(value: UserHost) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<UserHost> for RawMessage {
    fn from(value: UserHost) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for UserHost {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<UserHost, ClientMessageError> {
        if (1..=5).contains(&params.len()) {
            let mut nicknames = Vec::with_capacity(params.len());
            for (index, raw) in params.into_iter().map(String::from).enumerate() {
                match raw.parse::<Nickname>() {
                    Ok(nick) => nicknames.push(nick),
                    Err(source) => {
                        return Err(ClientMessageError::ParseParam {
                            index,
                            raw,
                            source: Box::new(source),
                        })
                    }
                }
            }
            Ok(UserHost { nicknames })
        } else {
            Err(ClientMessageError::ParamQty(
                ParameterListSizeError::Range {
                    min_requested: 1,
                    max_requested: 5,
                    received: params.len(),
                },
            ))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("UserHost takes 1 to 5 nicknames, but {0} were supplied")]
pub struct UserHostError(pub usize);
