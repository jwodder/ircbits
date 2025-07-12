use anyhow::Context;
use clap::Parser;
use ircnet::{
    client::{
        Client, ClientError,
        autoresponders::{CtcpQueryResponder, PingResponder},
        commands::{Login, LoginParams},
    },
    connect::codecs::MessageCodecError,
};
use irctext::{
    ClientMessage, FinalParam, Message, Payload, Reply, Source,
    clientmsgs::{Join, Quit},
    ctcp::{CtcpMessage, CtcpParams},
    formatting::StyledLine,
    types::{Channel, Nickname, Username},
};
use itertools::Itertools; // join
use std::fmt::Write;
use std::io::{IsTerminal, stderr};
use tokio::select;
use tracing::Level;
use tracing_subscriber::{filter::Targets, fmt::time::OffsetTime, prelude::*};

#[derive(Clone, Debug, Eq, Parser, PartialEq)]
struct Arguments {
    #[arg(short = 'n', long)]
    nickname: Nickname,

    #[arg(short = 'p', long)]
    password: FinalParam,

    #[arg(short = 'r', long)]
    realname: FinalParam,

    #[arg(short = 'u', long)]
    username: Username,

    #[arg(long)]
    tls: bool,

    #[arg(long)]
    trace: bool,

    server: String,

    port: u16,

    channels: Vec<Channel>,
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
                    .with_default(Level::INFO),
            )
            .init();
    }
    run(args)
}

#[tokio::main]
async fn run(args: Arguments) -> anyhow::Result<()> {
    let mut client = Client::connect(&args.server, args.port, args.tls).await?;
    client.add_autoresponder(PingResponder::new());
    client.add_autoresponder(
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
    );
    let login_output = client
        .run(Login::new(LoginParams {
            password: args.password,
            nickname: args.nickname,
            username: args.username,
            realname: args.realname,
        }))
        .await??;
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
    for channel in args.channels {
        client.send(Join::new_channel(channel).into()).await?;
    }
    loop {
        select! {
            r = client.recv() => {
                match r {
                    Ok(Some(msg)) => report(&format_msg(msg)),
                    Ok(None) => break,
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
            format!("[EVENT] {sender} joins {}", m.channels()[0])
        }
        Payload::ClientMessage(ClientMessage::Part(m)) => {
            let mut s = format!("[EVENT] {sender} leaves {}", m.channels()[0]);
            if let Some(txt) = m.reason() {
                write!(&mut s, ": {}", ircfmt_to_ansi(txt.as_str())).unwrap();
            }
            s
        }
        Payload::ClientMessage(ClientMessage::Quit(m)) => {
            let mut s = format!("[EVENT] {sender} quits");
            if let Some(txt) = m.reason() {
                write!(&mut s, ": {}", ircfmt_to_ansi(txt.as_str())).unwrap();
            }
            s
        }
        Payload::ClientMessage(ClientMessage::Error(m)) => {
            format!("[ERROR] {}", ircfmt_to_ansi(m.reason().as_str()))
        }
        Payload::ClientMessage(ClientMessage::Nick(m)) => {
            format!("[EVENT] {sender} is now known as {}", m.nickname())
        }
        Payload::ClientMessage(ClientMessage::Topic(m)) => {
            if let Some(topic) = m.topic() {
                format!(
                    "[EVENT] {sender} changed the {} topic: {}",
                    m.channel(),
                    ircfmt_to_ansi(topic.as_str())
                )
            } else {
                format!("[EVENT] {sender} unset the {} topic", m.channel())
            }
        }
        Payload::ClientMessage(ClientMessage::Invite(m)) => {
            format!(
                "[EVENT] {sender} invited {} to {}",
                m.nickname(),
                m.channel()
            )
        }
        Payload::ClientMessage(ClientMessage::Kick(m)) => {
            if let Some(cmt) = m.comment() {
                format!(
                    "[EVENT] {sender} kicked {} from {}: {}",
                    m.users()[0],
                    m.channel(),
                    ircfmt_to_ansi(cmt.as_str()),
                )
            } else {
                format!(
                    "[EVENT] {sender} kicked {} from {}",
                    m.users()[0],
                    m.channel()
                )
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
