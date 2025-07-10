use crate::codecs::{MessageCodec, RawMessageCodec};
use itertools::Itertools; // join
use rustls_pki_types::{InvalidDnsNameError, ServerName};
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_rustls::{
    TlsConnector,
    rustls::{ClientConfig, RootCertStore},
};
use tokio_util::{codec::Framed, either::Either};

pub type TlsStream = tokio_rustls::client::TlsStream<TcpStream>;

pub type Connection = Either<TcpStream, TlsStream>;

pub type RawMessageChannel = Framed<Connection, RawMessageCodec>;

pub type MessageChannel = Framed<Connection, MessageCodec>;

#[tracing::instrument]
pub async fn connect(server: &str, port: u16, tls: bool) -> Result<Connection, ConnectionError> {
    tracing::trace!("Connecting to remote server ...");
    let conn = TcpStream::connect((server, port))
        .await
        .map_err(ConnectionError::Connect)?;
    let addr = conn.peer_addr().ok().map(|addr| addr.to_string());
    tracing::trace!(remote_addr = addr, "Connected to remote server");
    if tls {
        tracing::trace!("Initializing TLS ...");
        let certs = rustls_native_certs::load_native_certs();
        if !certs.errors.is_empty() {
            let msg = certs.errors.into_iter().join("; ");
            return Err(ConnectionError::LoadStore(msg));
        }
        let mut root_cert_store = RootCertStore::empty();
        let (good, bad) = root_cert_store.add_parsable_certificates(certs.certs);
        if good == 0 {
            return Err(ConnectionError::AddCerts { bad });
        }
        let config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(config));
        let dnsname = ServerName::try_from(server)?.to_owned();
        let tls_conn = connector
            .connect(dnsname, conn)
            .await
            .map_err(ConnectionError::Tls)?;
        tracing::trace!("TLS established");
        Ok(Either::Right(tls_conn))
    } else {
        Ok(Either::Left(conn))
    }
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("failed to connect to server")]
    Connect(#[source] std::io::Error),
    #[error("failed to load system certificates: {0}")]
    LoadStore(String),
    #[error("failed to add certificates from system store: all {bad} certs were invalid")]
    AddCerts { bad: usize },
    #[error("invalid TLS server name")]
    ServerName(#[from] InvalidDnsNameError),
    #[error("failed to establish TLS connection")]
    Tls(#[source] std::io::Error),
}
