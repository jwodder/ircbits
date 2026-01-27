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
    pub fn new(nickname: &Nickname, password: &str) -> (PlainSasl, Authenticate) {
        let Ok(plain) = "PLAIN".parse::<TrailingParam>() else {
            unreachable!(r#""PLAIN" should be valid trailing param"#);
        };
        let mech_msg = Authenticate::new(plain);
        let auth_msgs = Authenticate::new_plain_sasl(nickname, nickname, password);
        (
            PlainSasl {
                state: State::AwaitingPlus { auth_msgs },
            },
            mech_msg,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum State {
    AwaitingPlus { auth_msgs: Vec<Authenticate> },
    Done,
    Void,
}

impl SaslFlow for PlainSasl {
    fn handle_message(&mut self, msg: Authenticate) -> Result<Vec<Authenticate>, SaslError> {
        replace_with_and_return(
            &mut self.state,
            || State::Void,
            |state| match state {
                State::AwaitingPlus { auth_msgs } => {
                    if msg.parameter() == "+" {
                        (Ok(auth_msgs), State::Done)
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
                State::Done => panic!("handle_message() called on Done SASL state"),
                State::Void => panic!("handle_message() called on Void SASL state"),
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
        let (mut flow, msg1) = PlainSasl::new(&"jwodder".parse::<Nickname>().unwrap(), "hunter2");
        assert_eq!(msg1.to_irc_line(), "AUTHENTICATE :PLAIN");
        let msg = Authenticate::new_empty();
        let outgoing = flow
            .handle_message(msg)
            .unwrap()
            .into_iter()
            .map(|msg| msg.to_irc_line())
            .collect::<Vec<_>>();
        assert_eq!(outgoing, ["AUTHENTICATE :andvZGRlcgBqd29kZGVyAGh1bnRlcjI="]);
        assert!(flow.is_done());
    }
}
