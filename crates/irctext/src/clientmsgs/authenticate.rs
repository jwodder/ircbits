use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{FinalParam, Message, ParameterList, RawMessage, Verb};
use base64::{Engine, engine::general_purpose::STANDARD};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Authenticate {
    parameter: FinalParam,
}

impl Authenticate {
    pub fn new(parameter: FinalParam) -> Authenticate {
        Authenticate { parameter }
    }

    pub fn new_encoded(bytes: &[u8]) -> Vec<Authenticate> {
        let mut b64 = STANDARD.encode(bytes);
        let mut msgs = Vec::with_capacity(b64.len() / 400 + 1);
        loop {
            if b64.is_empty() {
                let Ok(param) = "+".parse::<FinalParam>() else {
                    unreachable!(r#""+" should be valid final param"#);
                };
                msgs.push(Authenticate::new(param));
                return msgs;
            } else {
                let end = b64.len().min(400);
                let Ok(param) = FinalParam::try_from(b64.drain(..end).collect::<String>()) else {
                    unreachable!("base64 text should be valid final param");
                };
                msgs.push(Authenticate::new(param));
                if end < 400 && b64.is_empty() {
                    return msgs;
                }
            }
        }
    }

    pub fn new_plain_sasl(
        authorization: &str,
        authentication: &str,
        password: &str,
    ) -> Vec<Authenticate> {
        let s = format!("{authorization}\0{authentication}\0{password}");
        Authenticate::new_encoded(s.as_ref())
    }

    pub fn new_abort() -> Authenticate {
        let Ok(param) = "*".parse::<FinalParam>() else {
            unreachable!(r#""*" should be valid final param"#);
        };
        Authenticate::new(param)
    }

    pub fn parameter(&self) -> &FinalParam {
        &self.parameter
    }

    pub fn into_parameter(self) -> FinalParam {
        self.parameter
    }
}

impl ClientMessageParts for Authenticate {
    fn into_parts(self) -> (Verb, ParameterList) {
        (
            Verb::Authenticate,
            ParameterList::builder().with_final(self.parameter),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("AUTHENTICATE :{}", self.parameter)
    }
}

impl From<Authenticate> for Message {
    fn from(value: Authenticate) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Authenticate> for RawMessage {
    fn from(value: Authenticate) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Authenticate {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Authenticate, ClientMessageError> {
        let (parameter,) = params.try_into()?;
        Ok(Authenticate { parameter })
    }
}
