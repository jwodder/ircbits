use super::{SaslError, SaslFlow};
use irctext::{
    TrailingParam,
    clientmsgs::{Authenticate, ClientMessageParts},
    types::Nickname,
};
use replace_with::replace_with_and_return;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlainSasl {
    state: State,
}

impl PlainSasl {
    pub fn new(nickname: &Nickname, password: &str) -> PlainSasl {
        let Ok(plain) = "PLAIN".parse::<TrailingParam>() else {
            unreachable!(r#""PLAIN" should be valid trailing param"#);
        };
        let mech_msg = Authenticate::new(plain);
        let auth_msgs = Authenticate::new_plain_sasl(nickname, nickname, password);
        PlainSasl {
            state: State::Start {
                mech_msg,
                auth_msgs,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum State {
    Start {
        mech_msg: Authenticate,
        auth_msgs: Vec<Authenticate>,
    },
    AwaitingPlus {
        auth_msgs: Vec<Authenticate>,
    },
    GotPlus {
        auth_msgs: Vec<Authenticate>,
    },
    Done,
    Void,
}

impl SaslFlow for PlainSasl {
    fn handle_message(&mut self, msg: Authenticate) -> Result<(), SaslError> {
        replace_with_and_return(
            &mut self.state,
            || State::Void,
            |state| match state {
                State::Start { .. } => {
                    panic!("handle_message() called before calling get_output()")
                }
                State::AwaitingPlus { auth_msgs } => {
                    if msg.parameter() == "+" {
                        (Ok(()), State::GotPlus { auth_msgs })
                    } else {
                        (
                            Err(SaslError::Unexpected {
                                expecting: r#""AUTHENTICATE +""#,
                                msg: msg.to_irc_line(),
                            }),
                            State::Done,
                        )
                    }
                }
                State::GotPlus { .. } => {
                    panic!("handle_message() called before calling get_output()")
                }
                State::Done => panic!("handle_message() called on Done SASL state"),
                State::Void => panic!("handle_message() called on Void SASL state"),
            },
        )
    }

    fn get_output(&mut self) -> Vec<Authenticate> {
        replace_with_and_return(
            &mut self.state,
            || State::Void,
            |state| match state {
                State::Start {
                    mech_msg,
                    auth_msgs,
                } => (vec![mech_msg], State::AwaitingPlus { auth_msgs }),
                State::AwaitingPlus { .. } => (Vec::new(), state),
                State::GotPlus { auth_msgs } => (auth_msgs, State::Done),
                State::Done => (Vec::new(), State::Done),
                State::Void => panic!("get_output() called on Void SASL state"),
            },
        )
    }

    fn is_done(&self) -> bool {
        self.state == State::Done
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login() {
        let mut flow = PlainSasl::new(&"jwodder".parse::<Nickname>().unwrap(), "hunter2");
        let outgoing = flow
            .get_output()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["AUTHENTICATE :PLAIN"]);
        assert!(!flow.is_done());
        let msg = Authenticate::new_empty();
        assert!(flow.handle_message(msg).is_ok());
        let outgoing = flow
            .get_output()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["AUTHENTICATE :andvZGRlcgBqd29kZGVyAGh1bnRlcjI="]);
        assert!(flow.is_done());
    }
}
