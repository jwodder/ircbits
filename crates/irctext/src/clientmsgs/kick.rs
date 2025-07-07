use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::{join_with_commas, DisplayMaybeFinal};
use crate::{Channel, FinalParam, MedialParam, Message, Nickname, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Kick {
    channel: Channel,
    users: Vec<Nickname>,
    comment: Option<FinalParam>,
}

impl Kick {
    pub fn new(channel: Channel, user: Nickname) -> Kick {
        Kick {
            channel,
            users: vec![user],
            comment: None,
        }
    }

    pub fn new_with_comment(channel: Channel, user: Nickname, comment: FinalParam) -> Kick {
        Kick {
            channel,
            users: vec![user],
            comment: Some(comment),
        }
    }

    pub fn new_many<I: IntoIterator<Item = Nickname>>(channel: Channel, users: I) -> Option<Kick> {
        let users = users.into_iter().collect::<Vec<_>>();
        if users.is_empty() {
            None
        } else {
            Some(Kick {
                channel,
                users,
                comment: None,
            })
        }
    }

    pub fn new_many_with_comment<I: IntoIterator<Item = Nickname>>(
        channel: Channel,
        users: I,
        comment: FinalParam,
    ) -> Option<Kick> {
        let users = users.into_iter().collect::<Vec<_>>();
        if users.is_empty() {
            None
        } else {
            Some(Kick {
                channel,
                users,
                comment: Some(comment),
            })
        }
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn users(&self) -> &[Nickname] {
        &self.users
    }

    pub fn comment(&self) -> Option<&FinalParam> {
        self.comment.as_ref()
    }

    fn users_param(&self) -> MedialParam {
        assert!(
            !self.users.is_empty(),
            "Kick.users should always be nonempty"
        );
        let s = join_with_commas(&self.users);
        MedialParam::try_from(s).expect("comma-separated nicknames should be a valid MedialParam")
    }
}

impl ClientMessageParts for Kick {
    fn into_parts(self) -> (Verb, ParameterList) {
        let users_param = self.users_param();
        (
            Verb::Kick,
            ParameterList::builder()
                .with_medial(self.channel)
                .with_medial(users_param)
                .maybe_with_final(self.comment),
        )
    }

    fn to_irc_line(&self) -> String {
        format!(
            "KICK {} {}{}",
            self.channel,
            self.users_param(),
            DisplayMaybeFinal(self.comment.as_ref())
        )
    }
}

impl From<Kick> for Message {
    fn from(value: Kick) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Kick> for RawMessage {
    fn from(value: Kick) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Kick {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Kick, ClientMessageError> {
        let (p1, p2, comment): (_, _, Option<FinalParam>) = params.try_into()?;
        let channel = match p1.as_str().parse::<Channel>() {
            Ok(channel) => channel,
            Err(source) => {
                return Err(ClientMessageError::ParseParam {
                    index: 0,
                    raw: p1.into_inner(),
                    source: Box::new(source),
                })
            }
        };
        let users = match p2
            .as_str()
            .split(',')
            .map(str::parse::<Nickname>)
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(channels) => channels,
            Err(source) => {
                return Err(ClientMessageError::ParseParam {
                    index: 1,
                    raw: p2.into_inner(),
                    source: Box::new(source),
                })
            }
        };
        Ok(Kick {
            channel,
            users,
            comment,
        })
    }
}
