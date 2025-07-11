use crate::autoresponders::{AutoResponder, AutoResponderSet};
use crate::codecs::{MessageCodec, MessageCodecError};
use crate::consts::MAX_LINE_LENGTH;
use crate::{ConnectionError, MessageChannel, connect};
use futures_util::{SinkExt, TryStreamExt};
use irctext::{ClientMessage, Message};
use std::collections::VecDeque;
use tokio_util::codec::Framed;

#[allow(missing_debug_implementations)]
pub struct Client {
    channel: MessageChannel,
    autoresponders: AutoResponderSet,
    queued: VecDeque<ClientMessage>,
    recved: Option<Message>,
}

impl Client {
    pub async fn connect(server: &str, port: u16, tls: bool) -> Result<Client, ConnectionError> {
        let conn = connect(server, port, tls).await?;
        let codec = MessageCodec::new_with_max_length(MAX_LINE_LENGTH);
        let channel = Framed::new(conn, codec);
        let autoresponders = AutoResponderSet::new();
        Ok(Client {
            channel,
            autoresponders,
            queued: VecDeque::new(),
            recved: None,
        })
    }

    pub async fn send(&mut self, msg: ClientMessage) -> Result<(), MessageCodecError> {
        self.channel.send(msg).await
    }

    async fn flush_queue(&mut self) -> Result<(), MessageCodecError> {
        while let Some(msg) = self.queued.front().cloned() {
            let r = self.send(msg).await;
            let _ = self.queued.pop_front();
            r?;
        }
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Option<Message>, MessageCodecError> {
        loop {
            if let Some(msg) = self.recved.take() {
                return Ok(Some(msg));
            }
            self.flush_queue().await?;
            let r = self.channel.try_next().await?;
            if let Some(msg) = r {
                // Store outgoing client messages and the received message on
                // self in order to not lose data on cancellation
                let handled = self.autoresponders.handle_message(&msg);
                self.queued
                    .extend(self.autoresponders.get_client_messages());
                if !handled {
                    self.recved = Some(msg);
                }
                self.flush_queue().await?;
            } else {
                return Ok(None);
            }
        }
    }
}
