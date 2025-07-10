use super::Handler;
use irctext::{ClientMessage, Message};

#[allow(missing_debug_implementations)]
#[derive(Default)]
pub struct HandlerSet(Vec<Box<dyn Handler>>);

impl HandlerSet {
    pub fn new() -> HandlerSet {
        HandlerSet::default()
    }

    pub fn push<H: Handler + 'static>(&mut self, mut handler: H) -> Vec<ClientMessage> {
        let msgs = handler.get_client_messages();
        self.0.push(Box::new(handler));
        msgs
    }

    fn cleanup(&mut self) {
        self.0.retain(|h| !h.is_done());
    }
}

impl Handler for HandlerSet {
    fn get_client_messages(&mut self) -> Vec<ClientMessage> {
        let msgs = self
            .0
            .iter_mut()
            .flat_map(Handler::get_client_messages)
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
