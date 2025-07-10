use super::Handler;
use irctext::{clientmsgs::Pong, ClientMessage, Message, Payload};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PingHandler {
    pong: Option<Pong>,
}

impl PingHandler {
    pub fn new() -> PingHandler {
        PingHandler::default()
    }
}

impl Handler for PingHandler {
    fn get_client_messages(&mut self) -> Vec<ClientMessage> {
        Vec::from_iter(self.pong.take().map(ClientMessage::from))
    }

    fn handle_message(&mut self, msg: &Message) -> bool {
        if let Payload::ClientMessage(ClientMessage::Ping(ref ping)) = msg.payload {
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
