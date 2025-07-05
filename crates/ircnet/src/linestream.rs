use crate::client::MAX_LINE_LENGTH;
use crate::codec::IrcLinesCodec;
use anyhow::Context;
use rustls_pki_types::ServerName;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::{
    rustls::{ClientConfig, RootCertStore},
    TlsConnector,
};
use tokio_util::{codec::Framed, either::Either};

pub type IrcLineStream = Framed<Either<TcpStream, TlsStream>, IrcLinesCodec>;

pub type TlsStream = tokio_rustls::client::TlsStream<TcpStream>;

pub async fn connect(server: &str, port: u16, tls: bool) -> anyhow::Result<IrcLineStream> {
    log::trace!("Connecting to {server:?} on port {port} ...");
    let conn = TcpStream::connect((server, port))
        .await
        .context("Error connecting to server")?;
    log::trace!(
        "Connected to {}",
        conn.peer_addr().map_or_else(
            |_| String::from("<unknown peer address>"),
            |addr| addr.to_string()
        )
    );
    let conn = if tls {
        log::trace!("Initializing TLS ...");
        let mut root_cert_store = RootCertStore::empty();
        let system_certs = rustls_native_certs::load_native_certs()
            .context("Failed to load system certificate store")?;
        let (good, bad) = root_cert_store.add_parsable_certificates(system_certs);
        if good == 0 {
            anyhow::bail!(
                "Failed to load any certificates from system store: all {bad} certs were invalid"
            );
        }
        let config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(config));
        let dnsname = ServerName::try_from(server)
            .context("Invalid TLS server name")?
            .to_owned();
        let tls_conn = connector
            .connect(dnsname, conn)
            .await
            .context("Error establishing TLS connection")?;
        log::trace!("TLS established");
        Either::Right(tls_conn)
    } else {
        Either::Left(conn)
    };
    Ok(Framed::new(
        conn,
        IrcLinesCodec::new_with_max_length(MAX_LINE_LENGTH),
    ))
}
