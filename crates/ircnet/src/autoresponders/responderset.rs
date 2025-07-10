use super::AutoResponder;
use irctext::{ClientMessage, Message};

#[allow(missing_debug_implementations)]
#[derive(Default)]
pub struct AutoResponderSet(Vec<Box<dyn AutoResponder>>);

impl AutoResponderSet {
    pub fn new() -> AutoResponderSet {
        AutoResponderSet::default()
    }

    pub fn push<H: AutoResponder + 'static>(&mut self, handler: H) {
        self.0.push(Box::new(handler));
    }

    fn cleanup(&mut self) {
        self.0.retain(|h| !h.is_done());
    }
}

impl AutoResponder for AutoResponderSet {
    fn get_client_messages(&mut self) -> Vec<ClientMessage> {
        let msgs = self
            .0
            .iter_mut()
            .flat_map(AutoResponder::get_client_messages)
            .collect();
        self.cleanup();
        msgs
    }

    fn handle_message(&mut self, msg: &Message) -> bool {
        let mut handled = false;
        for h in &mut self.0 {
            if h.handle_message(msg) {
                handled = true;
            }
        }
        handled
    }

    fn is_done(&self) -> bool {
        self.0.is_empty()
    }
}
