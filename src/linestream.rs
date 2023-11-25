use crate::codec::IrcLinesCodec;
use anyhow::Context;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::{
    rustls::{ClientConfig, RootCertStore, ServerName},
    TlsConnector,
};
use tokio_util::{codec::Framed, either::Either};

pub(crate) type LineStream = Framed<Either<TcpStream, TlsStream>, IrcLinesCodec>;

pub(crate) type TlsStream = tokio_rustls::client::TlsStream<TcpStream>;

pub(crate) async fn connect(server: &str, port: u16, tls: bool) -> anyhow::Result<LineStream> {
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
            .context("Failed to load system certificate store")?
            .into_iter()
            .map(|cert| cert.0)
            .collect::<Vec<_>>();
        let (good, bad) = root_cert_store.add_parsable_certificates(&system_certs);
        if good == 0 {
            anyhow::bail!(
                "Failed to load any certificates from system store: all {bad} certs were invalid"
            );
        }
        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(config));
        let dnsname = ServerName::try_from(server).context("Invalid TLS server name")?;
        let tls_conn = connector
            .connect(dnsname, conn)
            .await
            .context("Error establishing TLS connection")?;
        log::trace!("TLS established");
        Either::Right(tls_conn)
    } else {
        Either::Left(conn)
    };
    // TODO: Set max line length
    Ok(Framed::new(conn, IrcLinesCodec::new()))
}
