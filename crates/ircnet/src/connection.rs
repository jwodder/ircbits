use crate::codec::IrcCodec;
use itertools::Itertools; // join
use rustls_pki_types::{InvalidDnsNameError, ServerName};
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_rustls::{
    rustls::{ClientConfig, RootCertStore},
    TlsConnector,
};
use tokio_util::{codec::Framed, either::Either};

pub const PLAIN_PORT: u16 = 6667;

pub const TLS_PORT: u16 = 6697;

// Both RFC 2812 and <https://modern.ircdocs.horse> say that IRC messages (when
// tags aren't involved) are limited to 512 characters, counting the CR LF.
pub const MAX_LINE_LENGTH: usize = 512;

pub type IrcConnection = Framed<Either<TcpStream, TlsStream>, IrcCodec>;

pub type TlsStream = tokio_rustls::client::TlsStream<TcpStream>;

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
    TlsConnect(#[source] std::io::Error),
}

pub async fn connect(server: &str, port: u16, tls: bool) -> Result<IrcConnection, ConnectionError> {
    log::trace!("Connecting to {server:?} on port {port} ...");
    let conn = TcpStream::connect((server, port))
        .await
        .map_err(ConnectionError::Connect)?;
    log::trace!(
        "Connected to {}",
        conn.peer_addr().map_or_else(
            |_| String::from("<unknown peer address>"),
            |addr| addr.to_string()
        )
    );
    let conn = if tls {
        log::trace!("Initializing TLS ...");
        let certs = rustls_native_certs::load_native_certs();
        if !certs.errors.is_empty() {
            let msg = certs.errors.into_iter().map(|e| e.to_string()).join("; ");
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
            .map_err(ConnectionError::TlsConnect)?;
        log::trace!("TLS established");
        Either::Right(tls_conn)
    } else {
        Either::Left(conn)
    };
    Ok(Framed::new(
        conn,
        IrcCodec::new_with_max_length(MAX_LINE_LENGTH),
    ))
}
