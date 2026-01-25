use anyhow::Context;
use clap::Parser;
use ircnet::client::{
    SessionBuilder, SessionParams,
    autoresponders::{CtcpQueryResponder, PingResponder},
    commands::{LuserStats, ServerInfo},
};
use irctext::{
    ClientMessage, Message, Payload, Reply, ReplyParts,
    clientmsgs::{Admin, Info, Links, Quit, Version},
    ctcp::CtcpParams,
    types::ISupportParam,
};
use mainutil::init_logging;
use patharg::OutputArg;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;
use tracing::Level;

const NEXT_REPLY_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, Eq, Parser, PartialEq)]
struct Arguments {
    /// Read IRC network connection details from the given configuration file
    #[arg(short = 'c', long, default_value = "ircbits.toml")]
    config: PathBuf,

    /// Output to the given path
    #[arg(short = 'o', long, default_value_t)]
    outfile: OutputArg,

    /// Select the profile in the configuration file to use
    #[arg(short = 'P', long, default_value = "irc")]
    profile: String,

    /// Emit log events for every message sent & received
    #[arg(long)]
    trace: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();
    let loglevel = if args.trace {
        Level::TRACE
    } else {
        Level::INFO
    };
    init_logging(env!("CARGO_CRATE_NAME"), loglevel);
    let cfgdata = std::fs::read(&args.config).context("failed to read configuration file")?;
    let mut cfg = toml::from_slice::<BTreeMap<String, SessionParams>>(&cfgdata)
        .context("failed to parse configuration file")?;
    let Some(profile) = cfg.remove(&args.profile) else {
        anyhow::bail!("{:?} profile not found in configuration file", args.profile);
    };

    tracing::info!("Connecting to IRC …");
    let (mut client, login_output) = SessionBuilder::new(profile)
        .with_autoresponder(PingResponder::new())
        .with_autoresponder(
            CtcpQueryResponder::new()
                .with_version(
                    env!("CARGO_CRATE_NAME")
                        .parse::<CtcpParams>()
                        .expect("Crate name should be valid CTCP params"),
                )
                .with_source(
                    env!("CARGO_PKG_REPOSITORY")
                        .parse::<CtcpParams>()
                        .expect("Project repository URL should be valid CTCP params"),
                ),
        )
        .build()
        .await?;
    tracing::info!("Connected");

    tracing::info!("Issuing VERSION query …");
    client.send(Version::new()).await?;
    let mut version = None;
    while let Ok(r) = timeout(NEXT_REPLY_TIMEOUT, client.recv()).await {
        let Some(Message { payload, .. }) = r? else {
            anyhow::bail!("Server suddenly disconnected");
        };
        match payload {
            Payload::Reply(Reply::Version(r)) => {
                version = Some(VersionInfo {
                    version: r.version().to_owned(),
                    server: r.server().to_owned(),
                    comments: r.comments().to_owned(),
                });
            }
            Payload::Reply(Reply::ISupport(_)) => (),
            Payload::ClientMessage(ClientMessage::Error(e)) => {
                anyhow::bail!("Server sent ERROR message: {:?}", e.reason())
            }
            Payload::ClientMessage(_) => (),
            Payload::Reply(r) if r.is_error() => {
                anyhow::bail!("Server returned error: {:?}", r.to_irc_line());
            }
            Payload::Reply(_) => (),
        }
    }
    if version.is_none() {
        tracing::info!("No RPL_VERSION reply received in time");
    }

    tracing::info!("Issuing ADMIN query …");
    client.send(Admin::new()).await?;
    let mut admin = AdminInfo::default();
    while let Ok(r) = timeout(NEXT_REPLY_TIMEOUT, client.recv()).await {
        let Some(Message { payload, .. }) = r? else {
            anyhow::bail!("Server suddenly disconnected");
        };
        match payload {
            Payload::Reply(Reply::AdminMe(_)) => (),
            Payload::Reply(Reply::AdminLoc1(r)) => admin.loc1 = Some(r.message().to_owned()),
            Payload::Reply(Reply::AdminLoc2(r)) => admin.loc2 = Some(r.message().to_owned()),
            Payload::Reply(Reply::AdminEmail(r)) => admin.email = Some(r.message().to_owned()),
            Payload::ClientMessage(ClientMessage::Error(e)) => {
                anyhow::bail!("Server sent ERROR message: {:?}", e.reason())
            }
            Payload::ClientMessage(_) => (),
            Payload::Reply(r) if r.is_error() => {
                anyhow::bail!("Server returned error: {:?}", r.to_irc_line());
            }
            Payload::Reply(_) => (),
        }
    }
    if admin == AdminInfo::default() {
        tracing::info!("No ADMIN replies received in time");
        // TODO: Set `admin` to `None`?
    }

    tracing::info!("Issuing LINKS query …");
    client.send(Links).await?;
    let mut links = Vec::new();
    loop {
        let Some(Message { payload, .. }) = client.recv().await? else {
            anyhow::bail!("Server suddenly disconnected");
        };
        match payload {
            Payload::Reply(Reply::Links(r)) => {
                links.push(Link {
                    server1: r.server1().to_owned(),
                    server2: r.server2().to_owned(),
                    hopcount: r.hopcount(),
                    server_info: r.server_info().to_owned(),
                });
            }
            Payload::Reply(Reply::EndOfLinks(_)) => break,
            Payload::ClientMessage(ClientMessage::Error(e)) => {
                anyhow::bail!("Server sent ERROR message: {:?}", e.reason())
            }
            Payload::ClientMessage(_) => (),
            // TODO: If we get ERR_UNKNOWNCOMMAND, set `links` to `None` somehow
            Payload::Reply(r) if r.is_error() => {
                anyhow::bail!("Server returned error: {:?}", r.to_irc_line());
            }
            Payload::Reply(_) => (),
        }
    }

    tracing::info!("Issuing INFO query …");
    client.send(Info).await?;
    let mut info = Vec::new();
    loop {
        let Some(Message { payload, .. }) = client.recv().await? else {
            anyhow::bail!("Server suddenly disconnected");
        };
        match payload {
            Payload::Reply(Reply::Info(r)) => {
                info.push(r.message().to_owned());
            }
            Payload::Reply(Reply::EndOfInfo(_)) => break,
            Payload::ClientMessage(ClientMessage::Error(e)) => {
                anyhow::bail!("Server sent ERROR message: {:?}", e.reason())
            }
            Payload::ClientMessage(_) => (),
            Payload::Reply(r) if r.is_error() => {
                anyhow::bail!("Server returned error: {:?}", r.to_irc_line());
            }
            Payload::Reply(_) => (),
        }
    }

    tracing::info!("Quitting …");
    client.send(Quit::new()).await?;
    while client.recv_new().await?.is_some() {}

    let capabilities = login_output.capabilities.map(|caps| {
        caps.into_iter()
            .map(|(name, value)| (String::from(name), value.map(String::from)))
            .collect::<BTreeMap<_, _>>()
    });
    let isupport = login_output
        .isupport
        .into_iter()
        .map(|s| match s {
            ISupportParam::Set(key) => (String::from(key), ISupportValue::Bool(true)),
            ISupportParam::Unset(key) => (String::from(key), ISupportValue::Bool(false)),
            ISupportParam::Eq(key, value) => {
                (String::from(key), ISupportValue::Str(String::from(value)))
            }
        })
        .collect::<BTreeMap<_, _>>();
    let output = IrcInfo {
        capabilities,
        server: login_output.server_info,
        isupport,
        lusers: login_output.luser_stats,
        motd: login_output.motd,
        version,
        admin,
        links,
        info,
    };

    let mut out = BufWriter::new(
        args.outfile
            .create()
            .context("failed to open output file")?,
    );
    serde_json::to_writer_pretty(&mut out, &output).context("failed to serialize output")?;
    out.write_all(b"\n")?;
    out.flush()?;
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct IrcInfo {
    capabilities: Option<BTreeMap<String, Option<String>>>,
    server: ServerInfo,
    isupport: BTreeMap<String, ISupportValue>,
    lusers: LuserStats,
    motd: Option<String>,
    version: Option<VersionInfo>,
    admin: AdminInfo,
    links: Vec<Link>,
    info: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
enum ISupportValue {
    Str(String),
    Bool(bool),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct VersionInfo {
    version: String,
    server: String,
    comments: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
struct AdminInfo {
    loc1: Option<String>,
    loc2: Option<String>,
    email: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct Link {
    server1: String,
    server2: String,
    hopcount: u32,
    server_info: String,
}
