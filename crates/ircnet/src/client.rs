use crate::autoresponders::{AutoResponder, AutoResponderSet};
use crate::codecs::{MessageCodec, MessageCodecError};
use crate::commands::Command;
use crate::consts::MAX_LINE_LENGTH;
use crate::{ConnectionError, MessageChannel, connect};
use futures_util::{SinkExt, TryStreamExt};
use irctext::{ClientMessage, Message};
use std::collections::VecDeque;
use thiserror::Error;
use tokio::time::{Instant, timeout_at};
use tokio_util::codec::Framed;

#[allow(missing_debug_implementations)]
pub struct Client {
    channel: MessageChannel,
    autoresponders: AutoResponderSet,
    queued: VecDeque<ClientMessage>,
    recved: Option<Message>,
}

impl Client {
    pub async fn connect(server: &str, port: u16, tls: bool) -> Result<Client, ClientError> {
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

    pub fn push_autoresponder<T: AutoResponder + Send + 'static>(&mut self, ar: T) {
        self.autoresponders.push(ar);
    }

    pub async fn send(&mut self, msg: ClientMessage) -> Result<(), ClientError> {
        self.channel.send(msg).await.map_err(ClientError::Send)
    }

    async fn flush_queue(&mut self) -> Result<(), ClientError> {
        while let Some(msg) = self.queued.front().cloned() {
            let r = self.send(msg).await;
            let _ = self.queued.pop_front();
            r?;
        }
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Option<Message>, ClientError> {
        loop {
            if let Some(msg) = self.recved.take() {
                return Ok(Some(msg));
            }
            self.flush_queue().await?;
            let r = self.channel.try_next().await.map_err(ClientError::Recv)?;
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

    pub async fn run<C: Command>(
        &mut self,
        mut cmd: C,
    ) -> Result<(C::Output, Vec<Message>), ClientError> {
        let mut unhandled = Vec::new();
        for climsg in cmd.get_client_messages() {
            self.send(climsg).await?;
        }
        let mut deadline = cmd.get_timeout().map(|d| Instant::now() + d);
        while !cmd.is_done() {
            let fut = self.recv();
            let r = if let Some(dl) = deadline {
                timeout_at(dl, fut).await.ok()
            } else {
                Some(fut.await)
            };
            match r {
                Some(Ok(None)) => return Err(ClientError::Disconnect),
                Some(Ok(Some(msg))) => {
                    if !cmd.handle_message(&msg) {
                        unhandled.push(msg);
                    }
                }
                Some(Err(e)) => return Err(e),
                None => {
                    deadline = None;
                    cmd.handle_timeout();
                }
            }
            for climsg in cmd.get_client_messages() {
                self.send(climsg).await?;
            }
            if let Some(d) = cmd.get_timeout() {
                deadline = Some(Instant::now() + d);
            }
        }
        Ok((cmd.get_output(), unhandled))
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("failed to connect to IRC server")]
    Connect(#[from] ConnectionError),
    #[error("failed send message to server")]
    Send(#[source] MessageCodecError),
    #[error("failed receive message from server")]
    Recv(#[source] MessageCodecError),
    #[error("connection terminated while running command")]
    Disconnect,
}
