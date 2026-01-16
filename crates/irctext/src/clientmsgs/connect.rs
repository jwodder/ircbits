use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::{Message, MiddleParam, ParameterList, ParameterListSizeError, RawMessage, Verb};
use std::fmt::Write;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Connect {
    target_server: MiddleParam,
    port: Option<u16>,
    remote_server: Option<MiddleParam>,
}

impl Connect {
    pub fn new(target_server: MiddleParam) -> Connect {
        Connect {
            target_server,
            port: None,
            remote_server: None,
        }
    }

    pub fn new_with_port(target_server: MiddleParam, port: u16) -> Connect {
        Connect {
            target_server,
            port: Some(port),
            remote_server: None,
        }
    }

    pub fn new_with_remote_server(
        target_server: MiddleParam,
        port: u16,
        remote_server: MiddleParam,
    ) -> Connect {
        Connect {
            target_server,
            port: Some(port),
            remote_server: Some(remote_server),
        }
    }

    pub fn target_server(&self) -> &MiddleParam {
        &self.target_server
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn remote_server(&self) -> Option<&MiddleParam> {
        self.remote_server.as_ref()
    }
}

impl ClientMessageParts for Connect {
    fn into_parts(self) -> (Verb, ParameterList) {
        let mut builder = ParameterList::builder().with_middle(self.target_server);
        if let Some(port) = self.port {
            let port_param = port
                .to_string()
                .parse::<MiddleParam>()
                .expect("stringified integer should be a valid MiddleParam");
            builder.push_middle(port_param);
            if let Some(remote) = self.remote_server {
                builder.push_middle(remote);
            }
        }
        (Verb::Connect, builder.finish())
    }

    fn to_irc_line(&self) -> String {
        let mut s = format!("CONNECT {}", self.target_server);
        if let Some(port) = self.port {
            let _ = write!(&mut s, " {port}");
            if let Some(ref remote) = self.remote_server {
                let _ = write!(&mut s, " {remote}");
            }
        }
        s
    }
}

impl From<Connect> for Message {
    fn from(value: Connect) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Connect> for RawMessage {
    fn from(value: Connect) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Connect {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Connect, ClientMessageError> {
        if (1..=3).contains(&params.len()) {
            let mut iter = params.into_iter();
            let p1 = iter
                .next()
                .expect("First element should exist when len >= 1");
            let target_server = MiddleParam::try_from(String::from(p1))?;
            let port = if let Some(p2) = iter.next() {
                match p2.as_str().parse::<u16>() {
                    Ok(p) => Some(p),
                    Err(inner) => {
                        return Err(ClientMessageError::Int {
                            string: p2.into(),
                            inner,
                        });
                    }
                }
            } else {
                None
            };
            let remote_server = if let Some(p3) = iter.next() {
                Some(MiddleParam::try_from(String::from(p3))?)
            } else {
                None
            };
            Ok(Connect {
                target_server,
                port,
                remote_server,
            })
        } else {
            Err(ClientMessageError::ParamQty(
                ParameterListSizeError::Range {
                    min_required: 1,
                    max_required: 3,
                    received: params.len(),
                },
            ))
        }
    }
}
