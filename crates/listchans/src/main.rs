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
use mainutil::init_logging;
use patharg::OutputArg;
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use tracing::Level;

/// Log into an IRC network, send a `LIST` command, output the response as
/// JSON, and disconnect.
///
/// Visit <https://github.com/jwodder/ircbits> for more information.
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
