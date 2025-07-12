use anyhow::Context;
use clap::Parser;
use ircnet::{
    client::{
        ClientError, SessionBuilder, SessionParams,
        autoresponders::{CtcpQueryResponder, PingResponder},
    },
    connect::codecs::MessageCodecError,
};
use irctext::{
    ClientMessage, FinalParam, Message, Payload, Reply, Source,
    clientmsgs::{Join, Quit},
    ctcp::{CtcpMessage, CtcpParams},
    formatting::StyledLine,
    types::Channel,
};
use itertools::Itertools; // join
use std::collections::HashMap;
use std::fmt::Write;
use std::io::{IsTerminal, stderr};
use std::path::PathBuf;
use tokio::select;
use tracing::Level;
use tracing_subscriber::{filter::Targets, fmt::time::OffsetTime, prelude::*};

#[derive(Clone, Debug, Eq, Parser, PartialEq)]
struct Arguments {
    #[arg(short = 'c', long, default_value = "ircbits.toml")]
    config: PathBuf,

    #[arg(short = 'P', long, default_value = "irc")]
    profile: String,

    #[arg(long)]
    trace: bool,

    channels: Vec<Channel>,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq)]
struct Profile {
    #[serde(flatten)]
    session_params: SessionParams,

    #[serde(default)]
    ircwatch: ProgramParams,
}

#[derive(Clone, Debug, Default, serde::Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct ProgramParams {
    #[serde(default)]
    default_channels: Vec<Channel>,
}

// See
// <https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/time/struct.OffsetTime.html#method.local_rfc_3339>
// for an explanation of the main + #[tokio::main]run thing
fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();
    if args.trace {
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
                    .with_target(env!("CARGO_CRATE_NAME"), Level::TRACE)
                    .with_target("ircnet", Level::TRACE)
                    .with_default(Level::INFO),
            )
            .init();
    }
    run(args)
}

#[tokio::main]
async fn run(args: Arguments) -> anyhow::Result<()> {
    let cfgdata = std::fs::read(&args.config).context("failed to read configuration file")?;
    let mut cfg = toml::from_slice::<HashMap<String, Profile>>(&cfgdata)
        .context("failed to parse configuration file")?;
    let Some(profile) = cfg.remove(&args.profile) else {
        anyhow::bail!("{:?} profile not found in configuration file", args.profile);
    };
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
    for msg in client.take_unhandled() {
        report(&format_msg(msg));
    }
    let mut login_msg = format!(
        "[LOGIN] Logged in as {}; server: {} (version: {})",
        login_output.my_nick,
        login_output.server_info.server_name,
        login_output.server_info.version
    );
    if let Some(ref ms) = login_output.mode {
        write!(&mut login_msg, "; user mode: {ms}").unwrap();
    }
    report(&login_msg);
    if let Some(ref motd) = login_output.motd {
        for ln in motd.lines() {
            report(&format!("[MOTD] {}", ircfmt_to_ansi(ln)));
        }
    } else {
        report("[NOMOTD] No Message of the Day set");
    }
    let channels = if args.channels.is_empty() {
        profile.ircwatch.default_channels
    } else {
        args.channels
    };
    for chan in channels {
        client.send(Join::new_channel(chan).into()).await?;
    }
    loop {
        select! {
            r = client.recv() => {
                match r {
                    Ok(Some(msg)) => report(&format_msg(msg)),
                    Ok(None) => {
                        report("* Disconnected");
                        break;
                    }
                    Err(ClientError::Recv(MessageCodecError::Parse(e))) => {
                        report(&format!("[PARSE FAILURE] {:?}", anyhow::Error::new(e)));
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            () = recv_stop_signal() => {
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

fn report(msg: &str) {
    let timestamp = jiff::Zoned::now()
        .time()
        .round(jiff::Unit::Second)
        .expect("Time.round(Second) should not fail");
    anstream::println!("[{timestamp}] {msg}");
}

fn format_msg(msg: Message) -> String {
    let sender = match &msg.source {
        Some(Source::Server(s)) => s.to_string(),
        Some(Source::Client(clisrc)) => clisrc.nickname.to_string(),
        None => String::from("<no source>"),
    };
    match msg.payload {
        Payload::ClientMessage(ClientMessage::PrivMsg(m)) => {
            let target = &m.targets()[0];
            if target.is_channel() {
                format!("[{}] {}", target, format_msgtext(&sender, m.text().clone()))
            } else {
                format!("[PRIVMSG] {}", format_msgtext(&sender, m.text().clone()))
            }
        }
        Payload::ClientMessage(ClientMessage::Notice(m)) => {
            let target = &m.targets()[0];
            if target.is_channel() {
                format!(
                    "[{}] [NOTICE] {}",
                    target,
                    format_msgtext(&sender, m.text().clone())
                )
            } else {
                format!("[NOTICE] {}", format_msgtext(&sender, m.text().clone()))
            }
        }
        Payload::ClientMessage(ClientMessage::Join(m)) => {
            format!("* {sender} joins {}", m.channels()[0])
        }
        Payload::ClientMessage(ClientMessage::Part(m)) => {
            let mut s = format!("* {sender} leaves {}", m.channels()[0]);
            if let Some(txt) = m.reason() {
                write!(&mut s, ": {}", ircfmt_to_ansi(txt.as_str())).unwrap();
            }
            s
        }
        Payload::ClientMessage(ClientMessage::Quit(m)) => {
            let mut s = format!("* {sender} quits");
            if let Some(txt) = m.reason() {
                write!(&mut s, ": {}", ircfmt_to_ansi(txt.as_str())).unwrap();
            }
            s
        }
        Payload::ClientMessage(ClientMessage::Error(m)) => {
            format!("[ERROR] {}", ircfmt_to_ansi(m.reason().as_str()))
        }
        Payload::ClientMessage(ClientMessage::Nick(m)) => {
            format!("* {sender} is now known as {}", m.nickname())
        }
        Payload::ClientMessage(ClientMessage::Topic(m)) => {
            if let Some(topic) = m.topic() {
                format!(
                    "* {sender} changed the {} topic: {}",
                    m.channel(),
                    ircfmt_to_ansi(topic.as_str())
                )
            } else {
                format!("* {sender} unset the {} topic", m.channel())
            }
        }
        Payload::ClientMessage(ClientMessage::Invite(m)) => {
            format!("* {sender} invited {} to {}", m.nickname(), m.channel())
        }
        Payload::ClientMessage(ClientMessage::Kick(m)) => {
            if let Some(cmt) = m.comment() {
                format!(
                    "* {sender} kicked {} from {}: {}",
                    m.users()[0],
                    m.channel(),
                    ircfmt_to_ansi(cmt.as_str()),
                )
            } else {
                format!("* {sender} kicked {} from {}", m.users()[0], m.channel())
            }
        }
        Payload::ClientMessage(ClientMessage::Wallops(m)) => {
            format!("[WALLOPS] {}", ircfmt_to_ansi(m.text()))
        }
        Payload::ClientMessage(_) => format!("[OTHER] Unexpected client message: {msg}"),
        Payload::Reply(Reply::NoTopic(r)) => {
            format!("[NOTOPIC] [{}] No topic set", r.channel())
        }
        Payload::Reply(Reply::Topic(r)) => {
            format!("[TOPIC] [{}] {}", r.channel(), ircfmt_to_ansi(r.topic()))
        }
        Payload::Reply(Reply::TopicWhoTime(r)) => {
            let who = r.user();
            let timestamp = fmt_unix_timestamp(r.setat());
            format!(
                "[TOPICWHO] [{}] Topic set at {timestamp} by {who}",
                r.channel()
            )
        }
        Payload::Reply(Reply::NamReply(r)) => {
            format!(
                "[MEMBERS] [{}] {}",
                r.channel(),
                r.clients().iter().map(|(_, nick)| nick).join(", ")
            )
        }
        Payload::Reply(Reply::EndOfNames(r)) => format!("[MEMBERS] [{}] [END]", r.channel()),
        Payload::Reply(_) => format!("[OTHER] Unexpected reply: {msg}"),
    }
}

fn format_msgtext(sender: &str, text: FinalParam) -> String {
    match CtcpMessage::from(text) {
        CtcpMessage::Plain(p) => format!("{sender}: {}", ircfmt_to_ansi(p.as_str())),
        CtcpMessage::Action(None) => format!("* {sender}"),
        CtcpMessage::Action(Some(p)) => format!("* {sender} {}", ircfmt_to_ansi(p.as_str())),
        // TODO: Should the following messages be parsed for IRC formatting?
        CtcpMessage::ClientInfo(optp) => fmt_ctcp(sender, "CLIENTINFO", optp),
        CtcpMessage::Dcc(optp) => fmt_ctcp(sender, "DCC", optp),
        CtcpMessage::Finger(optp) => fmt_ctcp(sender, "FINGER", optp),
        CtcpMessage::Ping(optp) => fmt_ctcp(sender, "PING", optp),
        CtcpMessage::Source(optp) => fmt_ctcp(sender, "SOURCE", optp),
        CtcpMessage::Time(optp) => fmt_ctcp(sender, "TIME", optp),
        CtcpMessage::UserInfo(optp) => fmt_ctcp(sender, "USERINFO", optp),
        CtcpMessage::Version(optp) => fmt_ctcp(sender, "VERSION", optp),
        CtcpMessage::Other { command, params } => fmt_ctcp(sender, command.as_str(), params),
    }
}

fn fmt_ctcp(sender: &str, cmd: &str, params: Option<CtcpParams>) -> String {
    let mut s = format!("[CTCP] {sender}: {cmd}");
    if let Some(p) = params {
        s.push(' ');
        s.push_str(p.as_str());
    }
    s
}

fn ircfmt_to_ansi(s: &str) -> String {
    StyledLine::parse(s).render_ansi().to_string()
}

fn fmt_unix_timestamp(ts: u64) -> String {
    let Ok(its) = i64::try_from(ts) else {
        return format!("@{ts}");
    };
    let Ok(jts) = jiff::Timestamp::from_second(its) else {
        return format!("@{ts}");
    };
    jts.to_string()
}
