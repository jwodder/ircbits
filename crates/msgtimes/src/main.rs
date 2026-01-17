use anyhow::Context;
use clap::Parser;
use either::Either;
use ircnet::client::{
    SessionBuilder, SessionParams,
    autoresponders::{CtcpQueryResponder, PingResponder},
    commands::JoinCommand,
};
use irctext::{
    CaseMapping, ClientMessage, Message, Payload, TrailingParam,
    clientmsgs::{Away, Quit},
    ctcp::CtcpParams,
    types::{Channel, ISupportParam, MsgTarget},
};
use mainutil::{init_logging, run_until_stopped};
use patharg::OutputArg;
use serde_jsonlines::JsonLinesWriter;
use std::collections::HashMap;
use std::io::{self, BufWriter};
use std::path::PathBuf;
use tracing::Level;

/// Log into an IRC network, join a given set of channels, and then run
/// indefinitely, outputting a timestamped JSON object for each `PRIVMSG` and
/// `NOTICE` message thereafter received.
///
/// Visit <https://github.com/jwodder/ircbits> for more information.
#[derive(Clone, Debug, Eq, Parser, PartialEq)]
struct Arguments {
    /// Read IRC network connection details from the given configuration file
    #[arg(short = 'c', long, default_value = "ircbits.toml")]
    config: PathBuf,

    /// Append output to the given path
    #[arg(short = 'o', long, default_value_t)]
    outfile: OutputArg,

    /// Select the profile in the configuration file to use
    #[arg(short = 'P', long, default_value = "irc")]
    profile: String,

    /// Emit log events for every message sent & received
    #[arg(long)]
    trace: bool,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq)]
struct Profile {
    #[serde(flatten)]
    session_params: SessionParams,

    #[serde(default)]
    msgtimes: ProgramParams,
}

#[derive(Clone, Debug, Default, serde::Deserialize, Eq, PartialEq)]
struct ProgramParams {
    channels: Vec<Channel>,
    away: Option<TrailingParam>,
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
    let mut cfg = toml::from_slice::<HashMap<String, Profile>>(&cfgdata)
        .context("failed to parse configuration file")?;
    let network = args.profile;
    let Some(profile) = cfg.remove(&network) else {
        anyhow::bail!("{network:?} profile not found in configuration file");
    };
    if profile.msgtimes.channels.is_empty() {
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

    if let Some(p) = profile.msgtimes.away {
        client.send(Away::new(p)).await?;
    }

    let mut canon_channels = ChannelCanonicalizer::new(casemapping);
    for chan in profile.msgtimes.channels {
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
        match run_until_stopped(client.recv()).await {
            Some(Ok(Some(Message {
                payload: Payload::ClientMessage(climsg),
                ..
            }))) => {
                match climsg {
                    ClientMessage::PrivMsg(m) => {
                        for t in m.targets() {
                            if let MsgTarget::Channel(c0) = t
                                && let Some(c) = canon_channels.get(c0).cloned()
                            {
                                log.log(Event::new(&network, Some(c.into_inner()), "message"))?;
                            }
                        }
                    }
                    ClientMessage::Notice(m) => {
                        for t in m.targets() {
                            if let MsgTarget::Channel(c0) = t
                                && let Some(c) = canon_channels.get(c0).cloned()
                            {
                                log.log(Event::new(&network, Some(c.into_inner()), "message"))?;
                            }
                        }
                    }
                    ClientMessage::Kick(m) => {
                        if let Some(chan) = canon_channels.get(m.channel())
                            && m.users()
                                .iter()
                                .any(|nick| casemapping.eq_ignore_case(nick, &me))
                        {
                            tracing::info!(
                                comment = m.comment().map(ToString::to_string),
                                "Kicked from {chan}"
                            );
                            log.log(Event::new(
                                &network,
                                Some(chan.as_str().to_owned()),
                                "kicked",
                            ))?;
                            let chan = chan.to_owned(); // Stop borrowing from canon_channels so we can mutate it
                            canon_channels.remove(&chan);
                            if canon_channels.is_empty() {
                                tracing::info!("No channels left; quitting");
                                client.send(Quit::new()).await?;
                            }
                        }
                    }
                    ClientMessage::Error(m) => {
                        tracing::info!(
                            reason = String::from(m.into_reason()),
                            "Server sent ERROR message"
                        );
                    }
                    _ => (),
                }
            }
            Some(Ok(Some(_))) => (),
            Some(Ok(None)) => {
                tracing::info!("Connection closed");
                log.log(Event::new(&network, None, "disconnected"))?;
                break;
            }
            Some(Err(e)) => {
                let e = anyhow::Error::new(e);
                tracing::error!(?e, "Error communicating with server");
                log.log(Event::new(&network, None, "error"))?;
                return Err(e);
            }
            None => {
                tracing::info!("Signal received; quitting");
                client
                    .send(Quit::new_with_reason(
                        "Terminated"
                            .parse::<TrailingParam>()
                            .expect(r#""Terminated" should be valid TrailingParam"#),
                    ))
                    .await?;
            }
        }
    }
    Ok(())
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
