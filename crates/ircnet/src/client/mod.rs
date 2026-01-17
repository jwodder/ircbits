//! High-level IRC client
pub mod autoresponders;
mod builder;
pub mod commands;
pub use self::autoresponders::AutoResponder;
use self::autoresponders::AutoResponderSet;
pub use self::builder::*;
pub use self::commands::Command;
use crate::connect::{
    ConnectionError, LinesChannel,
    codecs::{LinesCodec, LinesCodecError},
    connect,
    consts::{MAX_LINE_LENGTH_WITH_TAGS, PLAIN_PORT, TLS_PORT},
};
use futures_util::{SinkExt, TryStreamExt};
use irctext::{Message, ParseMessageError, TryFromStringError};
use std::collections::VecDeque;
use thiserror::Error;
use tokio::time::{Instant, timeout_at};
use tokio_util::codec::Framed;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ConnectionParams {
    pub host: String,
    pub port: Option<u16>,
    #[cfg_attr(feature = "serde", serde(default = "default_tls"))]
    pub tls: bool,
}

impl ConnectionParams {
    pub fn port(&self) -> u16 {
        match (self.port, self.tls) {
            (Some(p), _) => p,
            (None, true) => TLS_PORT,
            (None, false) => PLAIN_PORT,
        }
    }
}

#[cfg(feature = "serde")]
fn default_tls() -> bool {
    false
}

#[allow(missing_debug_implementations)]
pub struct Client {
    /// Name of remote host; used in log messages
    host: String,

    /// The TCP connection to the server, as a stream & sink of lines
    channel: LinesChannel,

    /// Set of `AutoResponder`s installed on this client
    autoresponders: AutoResponderSet,

    /// Outgoing client messages emitted by `autoresponders` that have not yet
    /// been sent to the server
    queued: VecDeque<Message>,

    /// A message received from the server that has not yet been returned to
    /// the caller, likely because the `recv()` method was cancelled while
    /// sending back autoresponses
    recved: Option<Message>,

    /// Messages received during execution of a `Command` that were not handled
    /// by the command
    unhandled: VecDeque<Message>,
}

impl Client {
    /// Create a new `Client` connected to the given server & port.  If `tls`
    /// is true, the connection will use SSL/TLS.
    pub async fn connect(params: ConnectionParams) -> Result<Client, ClientError> {
        let conn = connect(&params.host, params.port(), params.tls).await?;
        let codec = LinesCodec::new_with_max_length(MAX_LINE_LENGTH_WITH_TAGS);
        let channel = Framed::new(conn, codec);
        let autoresponders = AutoResponderSet::new();
        Ok(Client {
            host: params.host,
            channel,
            autoresponders,
            queued: VecDeque::new(),
            recved: None,
            unhandled: VecDeque::new(),
        })
    }

    /// Install the given `AutoResponder` in the client
    pub fn add_autoresponder<T: AutoResponder + Send + 'static>(&mut self, ar: T) {
        self.autoresponders.push(ar);
    }

    fn set_autoresponders(&mut self, set: AutoResponderSet) {
        self.autoresponders = set;
    }

    /// Send a client message to the server.
    ///
    /// # Cancellation safety
    ///
    /// If this method is cancelled, it is guaranteed that the message was not
    /// sent, but the message itself is lost.
    pub async fn send<M: Into<Message>>(&mut self, msg: M) -> Result<(), ClientError> {
        let line = msg.into().to_string();
        tracing::trace!(host = self.host, line, "Sending message to remote server");
        self.channel.send(line).await.map_err(ClientError::Send)
    }

    /// Receive the next message from the server that is not handled by an
    /// autoresponder.  Any messages emitted by an autoresponder in response to
    /// a received message will be sent before returning.
    ///
    /// If a previous call to `run()` received any messages that were not
    /// handled by the command, this method will first return each of the
    /// unhandled messages before receiving any new messages from the server.
    /// Use `recv_new()` to bypass this or use `take_unhandled()` to obtain all
    /// unhandled messages at once.
    ///
    /// # Cancellation safety
    ///
    /// If this method is cancelled, any messages emitted by autoresponders
    /// that were not sent will be preserved and will be sent on the next call
    /// to `recv()`.  If this method is cancelled after receiving a message but
    /// before sending all messages emitted by autoresponders, the message will
    /// be preserved and will be returned on the next call to `recv()`.
    pub async fn recv(&mut self) -> Result<Option<Message>, ClientError> {
        if let Some(msg) = self.unhandled.pop_front() {
            Ok(Some(msg))
        } else {
            self.recv_new().await
        }
    }

    /// Receive the next message from the server that is not handled by an
    /// autoresponder.  Any messages emitted by an autoresponder in response to
    /// a received message will be sent before returning.
    ///
    /// Unlike `recv()`, this will not return any messages left unhandled by a
    /// previous `run()` call.
    ///
    /// # Cancellation safety
    ///
    /// If this method is cancelled, any messages emitted by autoresponders
    /// that were not sent will be preserved and will be sent on the next call
    /// to `recv_new()`.  If this method is cancelled after receiving a message
    /// but before sending all messages emitted by autoresponders, the message
    /// will be preserved and will be returned on the next call to
    /// `recv_new()`.
    pub async fn recv_new(&mut self) -> Result<Option<Message>, ClientError> {
        loop {
            self.flush_queue().await?;
            if let Some(msg) = self.recved.take() {
                return Ok(Some(msg));
            }
            let r = self.channel.try_next().await.map_err(ClientError::Recv)?;
            if let Some(line) = r {
                tracing::trace!(
                    host = self.host,
                    line,
                    "Received message from remote server"
                );
                let msg = Message::try_from(line)?;
                // Store outgoing client messages and the received message on
                // self in order to not lose data on cancellation
                let handled = self.autoresponders.handle_message(&msg);
                self.queued
                    .extend(self.autoresponders.get_outgoing_messages());
                if !handled {
                    self.recved = Some(msg);
                }
                self.flush_queue().await?;
            } else {
                return Ok(None);
            }
        }
    }

    /// [Private] Send any queued outgoing client messages emitted by
    /// `autoresponders`.
    ///
    /// # Cancellation safety
    ///
    /// This method is cancellation-safe.
    async fn flush_queue(&mut self) -> Result<(), ClientError> {
        while let Some(msg) = self.queued.front() {
            let line = msg.to_string();
            tracing::trace!(
                host = self.host,
                line,
                "Sending autoresponse to remote server"
            );
            let r = self.channel.send(line).await.map_err(ClientError::Send);
            let _ = self.queued.pop_front();
            r?;
        }
        Ok(())
    }

    /// Retrieve all messages left unhandled by previous `run()` calls that
    /// have not yet been returned by `recv()`, preventing them from being
    /// returned by a later call to `recv()`
    pub fn take_unhandled(&mut self) -> VecDeque<Message> {
        std::mem::take(&mut self.unhandled)
    }

    /// Run a `Command` to completion, sending scripted client messages and
    /// handling replies, and return the command's output.
    ///
    /// During execution, if sending or receiving any messages fails or the
    /// connection is terminated, the command will be dropped and an error
    /// returned.
    ///
    /// A command may mark any number of messages received during execution as
    /// "handled," meaning that they will not be returned by future calls to
    /// `recv()` or `recv_new()`.  Any messages not marked handled will be
    /// returned by future calls to `recv()` but not `recv_new()`.
    ///
    /// # Cancellation safety
    ///
    /// This method is not cancellation-safe.
    pub async fn run<C: Command>(&mut self, mut cmd: C) -> Result<C::Output, ClientError> {
        for climsg in cmd.get_client_messages() {
            self.send(climsg).await?;
        }
        let mut deadline = cmd.get_timeout().map(|d| Instant::now() + d);
        while !cmd.is_done() {
            let fut = self.recv_new();
            let r = if let Some(dl) = deadline {
                timeout_at(dl, fut).await.ok()
            } else {
                Some(fut.await)
            };
            match r {
                Some(Ok(None)) => return Err(ClientError::Disconnect),
                Some(Ok(Some(msg))) => {
                    if !cmd.handle_message(&msg) {
                        self.unhandled.push_back(msg);
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
        cmd.get_output()
            .map_err(|e| ClientError::Command(Box::new(e)))
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("failed to connect to IRC server")]
    Connect(#[from] ConnectionError),
    #[error("failed to send message to server")]
    Send(#[source] LinesCodecError),
    #[error("failed to receive message from server")]
    Recv(#[source] LinesCodecError),
    #[error("failed to parse incoming message")]
    Parse(#[from] TryFromStringError<ParseMessageError>),
    #[error("command failed")]
    Command(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("connection terminated while running command")]
    Disconnect,
}
