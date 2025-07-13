use super::autoresponders::{AutoResponder, AutoResponderSet};
use super::commands::{Login, LoginOutput, LoginParams};
use super::{Client, ClientError, ConnectionParams};
use tracing::Instrument;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SessionParams {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub connect: ConnectionParams,

    #[cfg_attr(feature = "serde", serde(flatten))]
    pub login: LoginParams,
}

#[allow(missing_debug_implementations)]
pub struct SessionBuilder {
    connect: ConnectionParams,
    login: LoginParams,
    autoresponders: AutoResponderSet,
}

impl SessionBuilder {
    pub fn new(params: SessionParams) -> SessionBuilder {
        SessionBuilder {
            connect: params.connect,
            login: params.login,
            autoresponders: AutoResponderSet::new(),
        }
    }

    pub fn with_autoresponder<T: AutoResponder + Send + 'static>(mut self, ar: T) -> Self {
        self.autoresponders.push(ar);
        self
    }

    pub async fn build(self) -> Result<(Client, LoginOutput), ClientError> {
        let host = self.connect.host.clone();
        let mut client = Client::connect(self.connect).await?;
        client.set_autoresponders(self.autoresponders);
        let span = tracing::info_span!("login", host, nickname = self.login.nickname.as_str());
        let login_output = async {
            tracing::info!("Logging in to IRC network …");
            let r = client.run(Login::new(self.login)).await;
            if r.is_ok() {
                tracing::info!("Login successful");
            }
            r
        }
        .instrument(span)
        .await?;
        Ok((client, login_output))
    }
}
