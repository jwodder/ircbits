use super::AutoResponder;
use irctext::{clientmsgs::Pong, ClientMessage, Message, Payload};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PingResponder {
    pong: Option<Pong>,
}

impl PingResponder {
    pub fn new() -> PingResponder {
        PingResponder::default()
    }
}

impl AutoResponder for PingResponder {
    fn get_client_messages(&mut self) -> Vec<ClientMessage> {
        Vec::from_iter(self.pong.take().map(ClientMessage::from))
    }

    fn handle_message(&mut self, msg: &Message) -> bool {
        if let Payload::ClientMessage(ClientMessage::Ping(ref ping)) = msg.payload {
            tracing::trace!("PING received; queuing PONG");
            self.pong = Some(ping.to_pong());
            true
        } else {
            false
        }
    }

    fn is_done(&self) -> bool {
        false
    }
}
