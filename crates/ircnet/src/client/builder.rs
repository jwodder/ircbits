use super::autoresponders::{AutoResponder, AutoResponderSet};
use super::commands::{Login, LoginError, LoginOutput, LoginParams};
use super::{Client, ClientError, ConnectionParams};
use thiserror::Error;

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

    pub async fn build(self) -> Result<(Client, LoginOutput), SessionBuildError> {
        let mut client = Client::connect(self.connect).await?;
        client.set_autoresponders(self.autoresponders);
        let login_output = client.run(Login::new(self.login)).await??;
        Ok((client, login_output))
    }
}

#[derive(Debug, Error)]
pub enum SessionBuildError {
    #[error(transparent)]
    Client(#[from] ClientError),
    #[error(transparent)]
    Login(#[from] LoginError),
}
