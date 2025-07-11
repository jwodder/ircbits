use crate::autoresponders::AutoResponderSet;
use crate::codecs::MessageCodec;
use crate::consts::MAX_LINE_LENGTH;
use crate::{ConnectionError, MessageChannel, connect};
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
}
