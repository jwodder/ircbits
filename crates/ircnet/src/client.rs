use crate::autoresponders::AutoResponderSet;
use crate::codecs::{MessageCodec, MessageCodecError};
use crate::consts::MAX_LINE_LENGTH;
use crate::{ConnectionError, MessageChannel, connect};
use futures_util::{SinkExt, TryStreamExt};
use irctext::{ClientMessage, Message};
use tokio_util::codec::Framed;

#[allow(missing_debug_implementations)]
pub struct Client {
    channel: MessageChannel,
    autoresponders: AutoResponderSet,
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
        })
    }

    pub async fn send(&mut self, msg: ClientMessage) -> Result<(), MessageCodecError> {
        self.channel.send(msg).await
    }

    pub async fn recv(&mut self) -> Result<Option<Message>, MessageCodecError> {
        self.channel.try_next().await
    }
}
