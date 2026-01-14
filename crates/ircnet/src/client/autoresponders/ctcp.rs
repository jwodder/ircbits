use super::AutoResponder;
use irctext::{
    ClientMessage, ClientSource, Message, Payload, Source,
    clientmsgs::Notice,
    ctcp::{CtcpMessage, CtcpParams},
};
use jiff::{Timestamp, Zoned, tz::TimeZone};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CtcpQueryResponder {
    outgoing: Vec<ClientMessage>,
    finger: Option<CtcpParams>,
    source: Option<CtcpParams>,
    userinfo: Option<CtcpParams>,
    version: Option<CtcpParams>,
    utc_time: bool,
}

impl CtcpQueryResponder {
    pub fn new() -> Self {
        CtcpQueryResponder::default()
    }

    pub fn with_finger(mut self, finger: CtcpParams) -> Self {
        self.finger = Some(finger);
        self
    }

    pub fn with_source(mut self, source: CtcpParams) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_userinfo(mut self, userinfo: CtcpParams) -> Self {
        self.userinfo = Some(userinfo);
        self
    }

    pub fn with_version(mut self, version: CtcpParams) -> Self {
        self.version = Some(version);
        self
    }

    pub fn with_utc_time(mut self, utc_time: bool) -> Self {
        self.utc_time = utc_time;
        self
    }
}

impl AutoResponder for CtcpQueryResponder {
    fn get_client_messages(&mut self) -> Vec<ClientMessage> {
        std::mem::take(&mut self.outgoing)
    }

    fn handle_message(&mut self, msg: &Message) -> bool {
        let Some(source) = &msg.source else {
            return false;
        };
        let Source::Client(ClientSource {
            nickname: sender, ..
        }) = source
        else {
            return false;
        };
        let Payload::ClientMessage(ClientMessage::PrivMsg(privmsg)) = &msg.payload else {
            return false;
        };
        let ctcp = CtcpMessage::from(privmsg.text().clone());
        let resp = match ctcp {
            CtcpMessage::ClientInfo(None) => {
                tracing::info!(
                    source = source.to_string(),
                    "Received CLIENTINFO CTCP query; responding ..."
                );
                let mut s = String::from("CLIENTINFO");
                if self.finger.is_some() {
                    s.push_str(" FINGER");
                }
                s.push_str(" PING");
                if self.source.is_some() {
                    s.push_str(" SOURCE");
                }
                s.push_str(" TIME");
                if self.userinfo.is_some() {
                    s.push_str(" USERINFO");
                }
                if self.version.is_some() {
                    s.push_str(" VERSION");
                }
                match CtcpParams::try_from(s) {
                    Ok(ps) => CtcpMessage::ClientInfo(Some(ps)),
                    Err(e) => {
                        tracing::warn!(
                            err = e.to_string(),
                            "Failed to convert CLIENTINFO response to CtcpParams"
                        );
                        return true;
                    }
                }
            }
            CtcpMessage::Finger(None) => {
                if let Some(info) = self.finger.clone() {
                    tracing::info!(
                        source = source.to_string(),
                        "Received FINGER CTCP query; responding ..."
                    );
                    CtcpMessage::Finger(Some(info))
                } else {
                    tracing::info!(
                        source = source.to_string(),
                        "Received FINGER CTCP query, but no response defined"
                    );
                    return true;
                }
            }
            m @ CtcpMessage::Ping(_) => {
                tracing::info!(
                    source = source.to_string(),
                    "Received PING CTCP query; responding ..."
                );
                m
            }
            CtcpMessage::Source(None) => {
                if let Some(info) = self.source.clone() {
                    tracing::info!(
                        source = source.to_string(),
                        "Received SOURCE CTCP query; responding ..."
                    );
                    CtcpMessage::Source(Some(info))
                } else {
                    tracing::info!(
                        source = source.to_string(),
                        "Received SOURCE CTCP query, but no response defined"
                    );
                    return true;
                }
            }
            CtcpMessage::Time(None) => {
                tracing::info!(
                    source = source.to_string(),
                    "Received TIME CTCP query; responding ..."
                );
                let now = if self.utc_time {
                    Timestamp::now().to_zoned(TimeZone::UTC)
                } else {
                    Zoned::now()
                };
                match jiff::fmt::rfc2822::to_string(&now) {
                    Ok(stamp) => match CtcpParams::try_from(stamp) {
                        Ok(ps) => CtcpMessage::Time(Some(ps)),
                        Err(e) => {
                            tracing::warn!(
                                err = e.to_string(),
                                "Failed to convert TIME response to CtcpParams"
                            );
                            return true;
                        }
                    },
                    Err(e) => {
                        tracing::warn!(
                            err = e.to_string(),
                            timestamp = now.to_string(),
                            "Failed to format timestamp in RFC 2822 format"
                        );
                        return true;
                    }
                }
            }
            CtcpMessage::UserInfo(None) => {
                if let Some(info) = self.userinfo.clone() {
                    tracing::info!(
                        source = source.to_string(),
                        "Received USERINFO CTCP query; responding ..."
                    );
                    CtcpMessage::UserInfo(Some(info))
                } else {
                    tracing::info!(
                        source = source.to_string(),
                        "Received USERINFO CTCP query, but no response defined"
                    );
                    return true;
                }
            }
            CtcpMessage::Version(None) => {
                if let Some(info) = self.version.clone() {
                    tracing::info!(
                        source = source.to_string(),
                        "Received VERSION CTCP query; responding ..."
                    );
                    CtcpMessage::Version(Some(info))
                } else {
                    tracing::info!(
                        source = source.to_string(),
                        "Received VERSION CTCP query, but no response defined"
                    );
                    return true;
                }
            }
            _ => return false,
        };
        self.outgoing
            .push(Notice::new(sender.clone(), resp.into()).into());
        true
    }

    fn is_done(&self) -> bool {
        false
    }
}
