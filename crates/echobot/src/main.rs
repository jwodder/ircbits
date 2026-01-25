use anyhow::Context;
use clap::Parser;
use ircnet::client::{
    ClientError, SessionBuilder, SessionParams,
    autoresponders::{CtcpQueryResponder, PingResponder},
    commands::JoinCommand,
};
use irctext::{
    ClientMessage, Message, Payload, Source, TrailingParam,
    clientmsgs::{PrivMsg, Quit},
    ctcp::CtcpParams,
    types::{Channel, MsgTarget},
};
use mainutil::{ChannelSet, init_logging, recv_stop_signal};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::{task::JoinSet, time::sleep};
use tracing::Level;

const DEFAULT_ECHO_DELAY: Duration = Duration::from_secs(5);

/// IRC bot that sends back any messages sent to it after a delay
///
/// Visit <https://github.com/jwodder/ircbits> for more information.
#[derive(Clone, Debug, Eq, Parser, PartialEq)]
struct Arguments {
    /// Read IRC network connection details from the given configuration file
    #[arg(short = 'c', long, default_value = "ircbits.toml")]
    config: PathBuf,

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
    echobot: ProgramParams,
}

#[derive(Clone, Debug, Default, serde::Deserialize, Eq, PartialEq)]
struct ProgramParams {
    channels: Vec<Channel>,
    delay: Option<u64>,
}

impl ProgramParams {
    fn delay(&self) -> Duration {
        self.delay.map_or(DEFAULT_ECHO_DELAY, Duration::from_secs)
    }
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

    let casemapping = login_output.casemapping()?;
    let me = login_output.my_nick;
    let delay = profile.echobot.delay();
    let mut channels = ChannelSet::new(casemapping);
    for chan in profile.echobot.channels {
        tracing::info!("Joining {chan} …");
        let output = client.run(JoinCommand::new(chan.clone())).await?;
        let chan = output.channel;
        tracing::info!("Joined {chan}");
        channels.add(chan);
    }

    let mut pending = JoinSet::new();

    loop {
        let r = tokio::select! {
            r = client.recv() => Event::Recv(r),
            Some(r) = pending.join_next() => {
                let (reply_to, msg) = r.expect("Pending echo task should not have been cancelled or aborted");
                Event::EchoReady(reply_to, msg)
            }
            () = recv_stop_signal() => Event::Stopped,
        };
        match r {
            Event::Recv(Ok(Some(Message {
                source,
                payload: Payload::ClientMessage(climsg),
                ..
            }))) => {
                match climsg {
                    ClientMessage::PrivMsg(m) => {
                        if let Some(Source::Client(clisrc)) = source {
                            for t in m.targets() {
                                if let MsgTarget::Channel(c0) = t
                                    && let Some(c) = channels.canonicalize(c0)
                                    && let Some(msg) = strip_nick(&me, m.text())
                                {
                                    tracing::info!(msg, sender = %clisrc.nickname, channel = %c, "Received command message on channel");
                                    match msg.parse::<TrailingParam>() {
                                        Ok(echomsg) => {
                                            let reply_to = MsgTarget::from(c.clone());
                                            pending.spawn(async move {
                                                sleep(delay).await;
                                                (reply_to, echomsg)
                                            });
                                        }
                                        Err(_) => {
                                            tracing::warn!(
                                                ?msg,
                                                "Could not convert incoming message to something echoable"
                                            );
                                        }
                                    }
                                } else if casemapping.eq_ignore_case(t.as_str(), &me) {
                                    tracing::info!(msg = %m.text(), sender = %clisrc.nickname, "Received direct private message");
                                    let reply_to = MsgTarget::from(clisrc.nickname.clone());
                                    let echomsg = m.text().clone();
                                    pending.spawn(async move {
                                        sleep(delay).await;
                                        (reply_to, echomsg)
                                    });
                                }
                            }
                        }
                    }
                    ClientMessage::Kick(m) => {
                        if let Some(chan) = channels.canonicalize(m.channel())
                            && m.users()
                                .iter()
                                .any(|nick| casemapping.eq_ignore_case(nick, &me))
                        {
                            tracing::info!(
                                comment = m.comment().map(ToString::to_string),
                                "Kicked from {chan}"
                            );
                            let chan = chan.to_owned(); // Stop borrowing from `channels` so we can mutate it
                            channels.remove(&chan);
                            if channels.is_empty() {
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
            Event::Recv(Ok(Some(_))) => (),
            Event::Recv(Ok(None)) => {
                tracing::info!("Connection closed");
                break;
            }
            Event::Recv(Err(e)) => {
                let e = anyhow::Error::new(e);
                tracing::error!(?e, "Error communicating with server");
                return Err(e);
            }
            Event::Stopped => {
                tracing::info!("Signal received; quitting");
                client
                    .send(Quit::new_with_reason(
                        "Terminated"
                            .parse::<TrailingParam>()
                            .expect(r#""Terminated" should be valid TrailingParam"#),
                    ))
                    .await?;
            }
            Event::EchoReady(target, msg) => {
                tracing::info!(%target, %msg, "Sending message");
                client.send(PrivMsg::new(target, msg)).await?;
            }
        }
    }
    Ok(())
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum Event {
    Recv(Result<Option<Message>, ClientError>),
    EchoReady(MsgTarget, TrailingParam),
    Stopped,
}

fn strip_nick<'a>(nickname: &str, message: &'a str) -> Option<&'a str> {
    let (target, msg) = message.split_once(": ")?;
    let msg = msg.trim_start_matches(' ');
    (target.eq_ignore_ascii_case(nickname) && !msg.is_empty()).then_some(msg)
}
