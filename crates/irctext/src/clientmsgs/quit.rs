use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::util::DisplayMaybeFinal;
use crate::{FinalParam, Message, ParameterList, RawMessage, Verb};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Quit {
    reason: Option<FinalParam>,
}

impl Quit {
    pub fn new() -> Quit {
        Quit { reason: None }
    }

    pub fn new_with_reason(reason: FinalParam) -> Quit {
        Quit {
            reason: Some(reason),
        }
    }

    pub fn reason(&self) -> Option<&FinalParam> {
        self.reason.as_ref()
    }

    pub fn into_reason(self) -> Option<FinalParam> {
        self.reason
    }
}

impl ClientMessageParts for Quit {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Quit,
            ParameterList::builder().maybe_with_final(self.reason),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("QUIT{}", DisplayMaybeFinal(self.reason.as_ref()))
    }
}

impl From<Quit> for Message {
    fn from(value: Quit) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Quit> for RawMessage {
    fn from(value: Quit) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Quit {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Quit, ClientMessageError> {
        let (reason,) = params.try_into()?;
        Ok(Quit { reason })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Payload, Source};
    use assert_matches::assert_matches;

    #[test]
    fn parse_with_reason() {
        let msg = ":Spawns_Carpeting!~mobile@user/spawns-carpeting/x-6969421 QUIT :Quit: ZNC 1.8.2+deb3.1+deb12u1 - https://znc.in";
        let msg = msg.parse::<Message>().unwrap();
        assert_matches!(msg, Message {
            source: Some(Source::Client(clisrc)),
            payload: Payload::ClientMessage(ClientMessage::Quit(quit)),
        } => {
            assert_eq!(clisrc.nickname, "Spawns_Carpeting");
            assert_eq!(clisrc.user.as_ref().unwrap(), "~mobile");
            assert_eq!(clisrc.host.as_ref().unwrap(), "user/spawns-carpeting/x-6969421");
            assert_eq!(quit.reason().unwrap(), "Quit: ZNC 1.8.2+deb3.1+deb12u1 - https://znc.in");
        });
    }

    #[test]
    fn parse_no_reason() {
        let msg = ":Spawns_Carpeting!~mobile@user/spawns-carpeting/x-6969421 QUIT";
        let msg = msg.parse::<Message>().unwrap();
        assert_matches!(msg, Message {
            source: Some(Source::Client(clisrc)),
            payload: Payload::ClientMessage(ClientMessage::Quit(quit)),
        } => {
            assert_eq!(clisrc.nickname, "Spawns_Carpeting");
            assert_eq!(clisrc.user.as_ref().unwrap(), "~mobile");
            assert_eq!(clisrc.host.as_ref().unwrap(), "user/spawns-carpeting/x-6969421");
            assert_eq!(quit.reason(), None);
        });
    }
}
