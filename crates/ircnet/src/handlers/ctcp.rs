use super::Handler;
use irctext::{
    clientmsgs::Notice, ClientMessage, ClientSource, CtcpMessage, CtcpParams, Message, Payload,
    Source,
};
use time::{format_description::well_known::Rfc2822, OffsetDateTime, UtcOffset};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CtcpQueryHandler {
    outgoing: Vec<ClientMessage>,
    finger: Option<CtcpParams>,
    source: Option<CtcpParams>,
    version: Option<CtcpParams>,
    userinfo: Option<CtcpParams>,
    timezone: Option<UtcOffset>,
}

impl CtcpQueryHandler {
    pub fn new() -> Self {
        CtcpQueryHandler::default()
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

    pub fn with_timezone(mut self, timezone: UtcOffset) -> Self {
        self.timezone = Some(timezone);
        self
    }
}

impl Handler for CtcpQueryHandler {
    fn get_client_messages(&mut self) -> Vec<ClientMessage> {
        std::mem::take(&mut self.outgoing)
    }

    fn handle_message(&mut self, msg: &Message) -> bool {
        let Some(Source::Client(ClientSource {
            nickname: sender, ..
        })) = &msg.source
        else {
            return false;
        };
        let sender = sender.clone();
        let Payload::ClientMessage(ClientMessage::PrivMsg(privmsg)) = &msg.payload else {
            return false;
        };
        let ctcp = CtcpMessage::from(privmsg.text().clone());
        let resp = match ctcp {
            CtcpMessage::ClientInfo(None) => {
                let mut s = String::from("CLIENTINFO");
                if self.finger.is_some() {
                    s.push_str(" FINGER");
                }
                s.push_str(" PING");
                if self.source.is_some() {
                    s.push_str(" SOURCE");
                }
                if self.timezone.is_some() {
                    s.push_str(" TIME");
                }
                if self.userinfo.is_some() {
                    s.push_str(" USERINFO");
                }
                if self.version.is_some() {
                    s.push_str(" VERSION");
                }
                let ps =
                    CtcpParams::try_from(s).expect("CLIENTINFO params should be valid CtcpParams");
                Some(CtcpMessage::ClientInfo(Some(ps)))
            }
            CtcpMessage::Finger(None) => self
                .finger
                .clone()
                .map(|info| CtcpMessage::Finger(Some(info))),
            m @ CtcpMessage::Ping(_) => Some(m),
            CtcpMessage::Source(None) => self
                .source
                .clone()
                .map(|info| CtcpMessage::Source(Some(info))),
            CtcpMessage::Time(None) => {
                if let Some(tz) = self.timezone {
                    if let Ok(stamp) = OffsetDateTime::now_utc().to_offset(tz).format(&Rfc2822) {
                        let ps = CtcpParams::try_from(stamp)
                            .expect("RFC 2822 timestamp should be valid CtcpParams");
                        Some(CtcpMessage::Time(Some(ps)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            CtcpMessage::UserInfo(None) => self
                .userinfo
                .clone()
                .map(|info| CtcpMessage::UserInfo(Some(info))),
            CtcpMessage::Version(None) => self
                .version
                .clone()
                .map(|info| CtcpMessage::Version(Some(info))),
            _ => None,
        };
        if let Some(resp) = resp {
            self.outgoing.push(Notice::new(sender, resp.into()).into());
            true
        } else {
            false
        }
    }

    fn is_done(&self) -> bool {
        false
    }
}
