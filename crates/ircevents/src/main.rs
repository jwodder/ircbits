use anyhow::Context;
use clap::Parser;
use either::Either;
use ircnet::client::{
    SessionBuilder, SessionParams,
    autoresponders::{CtcpQueryResponder, PingResponder},
    commands::JoinCommand,
};
use irctext::{
    CaseMapping, ClientMessage, FinalParam, Message, Payload,
    clientmsgs::{Away, Quit},
    ctcp::CtcpParams,
    types::{Channel, ISupportParam, MsgTarget},
};
use patharg::OutputArg;
use serde_jsonlines::JsonLinesWriter;
use std::collections::HashMap;
use std::io::{self, BufWriter, IsTerminal, stderr};
use std::path::PathBuf;
use tokio::select;
use tracing::Level;
use tracing_subscriber::{filter::Targets, fmt::time::OffsetTime, prelude::*};

#[derive(Clone, Debug, Eq, Parser, PartialEq)]
struct Arguments {
    #[arg(short = 'c', long, default_value = "ircbits.toml")]
    config: PathBuf,

    #[arg(short = 'o', long, default_value_t)]
    outfile: OutputArg,

    #[arg(short = 'P', long, default_value = "irc")]
    profile: String,

    #[arg(long)]
    trace: bool,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq)]
struct Profile {
    #[serde(flatten)]
    session_params: SessionParams,

    #[serde(default)]
    ircevents: ProgramParams,
}

#[derive(Clone, Debug, Default, serde::Deserialize, Eq, PartialEq)]
struct ProgramParams {
    channels: Vec<Channel>,
    away: Option<FinalParam>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();

    let loglevel = if args.trace {
        Level::TRACE
    } else {
        Level::INFO
    };
    let timer =
        OffsetTime::local_rfc_3339().context("failed to determine local timezone offset")?;
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_timer(timer)
                .with_ansi(stderr().is_terminal())
                .with_writer(stderr),
        )
        .with(
            Targets::new()
                .with_target(env!("CARGO_CRATE_NAME"), loglevel)
                .with_target("ircnet", loglevel)
                .with_default(Level::INFO),
        )
        .init();

    let cfgdata = std::fs::read(&args.config).context("failed to read configuration file")?;
    let mut cfg = toml::from_slice::<HashMap<String, Profile>>(&cfgdata)
        .context("failed to parse configuration file")?;
    let network = args.profile;
    let Some(profile) = cfg.remove(&network) else {
        anyhow::bail!("{network:?} profile not found in configuration file");
    };
    if profile.ircevents.channels.is_empty() {
        anyhow::bail!("No channels configured for profile {network:?}");
    }

    let outfile = match args.outfile {
        OutputArg::Stdout => Either::Left(io::stdout().lock()),
        OutputArg::Path(p) => Either::Right(
            std::fs::File::options()
                .create(true)
                .append(true)
                .open(p)
                .context("failed to open output file")?,
        ),
    };
    let mut log = EventLogger::new(outfile);

    tracing::info!("Connecting to IRC …");
    let (mut client, login_output) = SessionBuilder::new(profile.session_params)
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

    let casemapping = login_output
        .isupport
        .iter()
        .find_map(|param| {
            if let ISupportParam::Eq(key, value) = param
                && key == "CASEMAPPING"
                && let Ok(cm) = value.as_str().parse::<CaseMapping>()
            {
                Some(cm)
            } else {
                None
            }
        })
        .unwrap_or_default();
    let me = login_output.my_nick;

    if let Some(p) = profile.ircevents.away {
        client.send(Away::new(p).into()).await?;
    }

    let mut canon_channels = ChannelCanonicalizer::new(casemapping);
    for chan in profile.ircevents.channels {
        tracing::info!("Joining {chan} …");
        let output = client.run(JoinCommand::new(chan.clone())).await?;
        let chan = output.channel;
        log.log(Event::new(
            &network,
            Some(chan.clone().into_inner()),
            "joined",
        ))?;
        canon_channels.add(chan);
    }

    loop {
        select! {
            r = client.recv() => {
                match r {
                    Ok(Some(Message {payload: Payload::ClientMessage(climsg), ..})) => {
                        match climsg {
                            ClientMessage::PrivMsg(m) => {
                                for t in m.targets() {
                                    if let MsgTarget::Channel(c0) = t && let Some(c) = canon_channels.get(c0).cloned() {
                                        log.log(Event::new(&network, Some(c.into_inner()), "message"))?;
                                    }
                                }
                            }
                            ClientMessage::Notice(m) => {
                                for t in m.targets() {
                                    if let MsgTarget::Channel(c0) = t && let Some(c) = canon_channels.get(c0).cloned() {
                                        log.log(Event::new(&network, Some(c.into_inner()), "message"))?;
                                    }
                                }
                            }
                            ClientMessage::Kick(m) => {
                                if let Some(chan) = canon_channels.get(m.channel()) && m.users().iter().any(|nick| nick == &me) {
                                    tracing::info!(comment = m.comment().map(ToString::to_string), "Kicked from {chan}");
                                    log.log(Event::new(&network, Some(chan.as_str().to_owned()), "kicked"))?;
                                    let chan = chan.to_owned(); // Stop borrowing from canon_channels so we can mutate it
                                    canon_channels.remove(&chan);
                                    if canon_channels.is_empty() {
                                        tracing::info!("No channels left; quitting");
                                        client.send(Quit::new().into()).await?;
                                    }
                                }
                            }
                            ClientMessage::Error(m) => {
                                tracing::info!("Server sent ERROR message: {}", m.reason());
                            }
                            _ => (),
                        }
                    }
                    Ok(Some(_)) => (),
                    Ok(None) => {
                        tracing::info!("Connection closed");
                        log.log(Event::new(&network, None, "disconnected"))?;
                        break;
                    }
                    Err(e) => {
                        let e = anyhow::Error::new(e);
                        tracing::error!(?e, "Error communicating with server");
                        log.log(Event::new(&network, None, "error"))?;
                        return Err(e);
                    }
                }
            }
            () = recv_stop_signal() => {
                tracing::info!("Signal received; quitting");
                client.send(Quit::new_with_reason("Terminated".parse::<FinalParam>().expect(r#""Terminated" should be valid FinalParam"#)).into()).await?;
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
async fn recv_stop_signal() -> () {
    use tokio::signal::unix::{SignalKind, signal};
    if let Ok(mut term) = signal(SignalKind::terminate()) {
        select! {
            _ = tokio::signal::ctrl_c() => (),
            _ = term.recv() => (),
        }
    } else {
        let _ = tokio::signal::ctrl_c().await;
    }
}

#[cfg(not(unix))]
async fn recv_stop_signal() -> () {
    let _ = tokio::signal::ctrl_c().await;
}

#[derive(Debug)]
struct EventLogger<W: io::Write>(JsonLinesWriter<BufWriter<W>>);

impl<W: io::Write> EventLogger<W> {
    fn new(writer: W) -> Self {
        EventLogger(JsonLinesWriter::new(BufWriter::new(writer)))
    }

    fn log(&mut self, event: Event) -> anyhow::Result<()> {
        self.0
            .write(&event)
            .context("failed to write event to log")?;
        self.0.flush().context("failed to write event to log")?;
        Ok(())
    }
}

#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
struct Event {
    network: String,
    channel: Option<String>,
    event: String,
    timestamp: String,
}

impl Event {
    fn new(network: &str, channel: Option<String>, event: &str) -> Event {
        let timestamp = jiff::Timestamp::now().to_string();
        Event {
            network: network.to_owned(),
            channel,
            event: event.to_owned(),
            timestamp,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ChannelCanonicalizer {
    casemapping: CaseMapping,
    lower2canon: HashMap<Channel, Channel>,
}

impl ChannelCanonicalizer {
    fn new(casemapping: CaseMapping) -> Self {
        Self {
            casemapping,
            lower2canon: HashMap::new(),
        }
    }

    fn add(&mut self, channel: Channel) {
        let lower = channel.to_lowercase(self.casemapping);
        self.lower2canon.insert(lower, channel);
    }

    fn get(&self, channel: &Channel) -> Option<&Channel> {
        let lower = channel.to_lowercase(self.casemapping);
        self.lower2canon.get(&lower)
    }

    fn remove(&mut self, channel: &Channel) {
        let lower = channel.to_lowercase(self.casemapping);
        self.lower2canon.remove(&lower);
    }

    fn is_empty(&self) -> bool {
        self.lower2canon.is_empty()
    }
}
