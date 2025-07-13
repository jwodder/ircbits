use anyhow::Context;
use clap::Parser;
use ircnet::client::{
    SessionBuilder, SessionParams,
    autoresponders::{CtcpQueryResponder, PingResponder},
    commands::ListCommand,
};
use irctext::{
    clientmsgs::{List, Quit},
    ctcp::CtcpParams,
};
use patharg::OutputArg;
use std::collections::HashMap;
use std::io::{self, BufWriter, IsTerminal, Write};
use std::path::PathBuf;
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

// See
// <https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/time/struct.OffsetTime.html#method.local_rfc_3339>
// for an explanation of the main + #[tokio::main]run thing
fn main() -> anyhow::Result<()> {
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
                .with_ansi(io::stderr().is_terminal())
                .with_writer(io::stderr),
        )
        .with(
            Targets::new()
                .with_target(env!("CARGO_CRATE_NAME"), loglevel)
                .with_target("ircnet", loglevel)
                .with_default(Level::INFO),
        )
        .init();
    run(args)
}

#[tokio::main]
async fn run(args: Arguments) -> anyhow::Result<()> {
    let cfgdata = std::fs::read(&args.config).context("failed to read configuration file")?;
    let mut cfg = toml::from_slice::<HashMap<String, SessionParams>>(&cfgdata)
        .context("failed to parse configuration file")?;
    let Some(profile) = cfg.remove(&args.profile) else {
        anyhow::bail!("{:?} profile not found in configuration file", args.profile);
    };
    tracing::info!("Connecting to IRC …");
    let (mut client, _) = SessionBuilder::new(profile)
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
    tracing::info!("Listing channels …");
    let output = client.run(ListCommand::new(List::new())).await?;
    tracing::info!("Quitting …");
    client.send(Quit::new().into()).await?;
    while client.recv_new().await?.is_some() {}
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
