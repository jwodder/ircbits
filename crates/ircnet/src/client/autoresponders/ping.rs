use super::AutoResponder;
use irctext::{ClientMessage, Message, Payload, clientmsgs::Pong};

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
    fn get_outgoing_messages(&mut self) -> Vec<Message> {
        Vec::from_iter(self.pong.take().map(Message::from))
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
