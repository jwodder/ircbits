mod highlight;
use crate::highlight::highlight;
use anyhow::Context;
use clap::Parser;
use ircnet::client::{
    ClientError, SessionBuilder, SessionParams,
    autoresponders::{CtcpQueryResponder, PingResponder},
    commands::JoinCommand,
};
use irctext::{
    ClientMessage, Message, Payload, Source, TrailingParam,
    clientmsgs::Quit,
    ctcp::{CtcpMessage, CtcpParams},
    formatting::StyledLine,
    types::{Channel, ChannelMembership},
};
use itertools::Itertools; // join
use mainutil::{init_logging, run_until_stopped};
use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;
use tracing::Level;

/// Log into an IRC network, join a given set of channels, and then
/// pretty-print all messages received to standard output until you hit Ctrl-C.
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

    /// Emit log events
    #[arg(long, overrides_with = "trace")]
    info: bool,

    /// Emit log events for every message sent & received
    #[arg(long)]
    trace: bool,

    /// Channels to join.  If not specified, the channels are taken from the
    /// configuration file.
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();
    let loglevel = if args.trace {
        Some(Level::TRACE)
    } else if args.info {
        Some(Level::INFO)
    } else {
        None
    };
    if let Some(loglevel) = loglevel {
        init_logging(env!("CARGO_CRATE_NAME"), loglevel);
    }
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
        login_output.my_nick, login_output.server_info.name, login_output.server_info.version
    );
    if let Some(ref ms) = login_output.mode {
        let _ = write!(&mut login_msg, "; user mode: {ms}");
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
        let hichan = highlight(&chan);
        let output = client.run(JoinCommand::new(chan.clone())).await?;
        report(&format!("[JOIN] Joined {hichan}"));
        if let Some(topic) = output.topic {
            report(&format!(
                "[JOIN] [{hichan}] Topic: {}",
                ircfmt_to_ansi(&topic)
            ));
            if let Some((setter, setat)) = output.topic_set_by.zip(output.topic_set_at) {
                let timestamp = fmt_unix_timestamp(setat);
                report(&format!(
                    "[JOIN] [{hichan}] Topic set at {timestamp} by {setter}"
                ));
            }
        } else {
            report(&format!("[JOIN] [{hichan}] No topic set"));
        }
        let mut s = format!(
            "[JOIN] [{hichan}] {status:?} channel",
            status = output.channel_status
        );
        let mut users = 0u32;
        let mut founders = 0u32;
        let mut protected = 0u32;
        let mut operators = 0u32;
        let mut halfops = 0u32;
        let mut voiced = 0u32;
        for (prefix, _) in output.users {
            users += 1;
            match prefix {
                Some(ChannelMembership::Founder) => founders += 1,
                Some(ChannelMembership::Protected) => protected += 1,
                Some(ChannelMembership::Operator) => operators += 1,
                Some(ChannelMembership::HalfOperator) => halfops += 1,
                Some(ChannelMembership::Voiced) => voiced += 1,
                _ => (),
            }
        }
        let _ = write!(&mut s, "; {users} users");
        if founders > 0 {
            let _ = write!(&mut s, ", {founders} founders");
        }
        if protected > 0 {
            let _ = write!(&mut s, ", {protected} protected");
        }
        if operators > 0 {
            let _ = write!(&mut s, ", {operators} operators");
        }
        if halfops > 0 {
            let _ = write!(&mut s, ", {halfops} halfops");
        }
        if voiced > 0 {
            let _ = write!(&mut s, ", {voiced} voiced");
        }
        report(&s);
    }
    loop {
        match run_until_stopped(client.recv()).await {
            Some(Ok(Some(msg))) => report(&format_msg(msg)),
            Some(Ok(None)) => {
                report("* Disconnected");
                break;
            }
            Some(Err(ClientError::Parse(e))) => {
                report(&format!("[PARSE FAILURE] {:?}", anyhow::Error::new(e)));
            }
            Some(Err(e)) => return Err(e.into()),
            None => {
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

fn report(msg: &str) {
    let timestamp = jiff::Zoned::now()
        .time()
        .round(jiff::Unit::Second)
        .expect("Time.round(Second) should not fail");
    anstream::println!("[{timestamp}] {msg}");
}

fn format_msg(msg: Message) -> String {
    let sender = match &msg.source {
        Some(Source::Server(s)) => highlight(&s.to_string()),
        Some(Source::Client(clisrc)) => highlight(&clisrc.nickname),
        None => String::from("<no source>"),
    };
    match msg.payload {
        Payload::ClientMessage(ClientMessage::PrivMsg(m)) => {
            let targets = m.targets();
            if targets.len() == 1 && targets[0].is_nick() {
                format!("[PRIVMSG] {}", format_msgtext(&sender, m.text().clone()))
            } else {
                format!(
                    "[{}] {}",
                    m.targets().iter().map(|t| highlight(t.as_str())).join(","),
                    format_msgtext(&sender, m.text().clone())
                )
            }
        }
        Payload::ClientMessage(ClientMessage::Notice(m)) => {
            let targets = m.targets();
            if targets.len() == 1 && targets[0].is_nick() {
                format!("[NOTICE] {}", format_msgtext(&sender, m.text().clone()))
            } else {
                format!(
                    "[{}] [NOTICE] {}",
                    m.targets().iter().map(|t| highlight(t.as_str())).join(","),
                    format_msgtext(&sender, m.text().clone())
                )
            }
        }
        Payload::ClientMessage(ClientMessage::Join(m)) => {
            format!(
                "* {sender} joins {}",
                join_and(m.channels().iter().map(|t| highlight(t.as_str())))
            )
        }
        Payload::ClientMessage(ClientMessage::Part(m)) => {
            let mut s = format!(
                "* {sender} leaves {}",
                join_and(m.channels().iter().map(|t| highlight(t.as_str())))
            );
            if let Some(txt) = m.reason() {
                let _ = write!(&mut s, ": {}", ircfmt_to_ansi(txt.as_str()));
            }
            s
        }
        Payload::ClientMessage(ClientMessage::Quit(m)) => {
            let mut s = format!("* {sender} quits");
            if let Some(txt) = m.reason() {
                let _ = write!(&mut s, ": {}", ircfmt_to_ansi(txt.as_str()));
            }
            s
        }
        Payload::ClientMessage(ClientMessage::Error(m)) => {
            format!("[ERROR] {}", ircfmt_to_ansi(m.reason().as_str()))
        }
        Payload::ClientMessage(ClientMessage::Nick(m)) => {
            format!("* {sender} is now known as {}", highlight(m.nickname()))
        }
        Payload::ClientMessage(ClientMessage::Topic(m)) => {
            if let Some(topic) = m.topic() {
                format!(
                    "* {sender} changed the {} topic: {}",
                    highlight(m.channel()),
                    ircfmt_to_ansi(topic.as_str())
                )
            } else {
                format!("* {sender} unset the {} topic", highlight(m.channel()))
            }
        }
        Payload::ClientMessage(ClientMessage::Invite(m)) => {
            format!(
                "* {sender} invited {} to {}",
                highlight(m.nickname()),
                highlight(m.channel())
            )
        }
        Payload::ClientMessage(ClientMessage::Kick(m)) => {
            if let Some(cmt) = m.comment() {
                format!(
                    "* {sender} kicked {} from {}: {}",
                    join_and(m.users().iter().map(|u| highlight(u))),
                    highlight(m.channel()),
                    ircfmt_to_ansi(cmt.as_str()),
                )
            } else {
                format!(
                    "* {sender} kicked {} from {}",
                    join_and(m.users().iter().map(|u| highlight(u))),
                    highlight(m.channel())
                )
            }
        }
        Payload::ClientMessage(ClientMessage::Wallops(m)) => {
            format!("[WALLOPS] {}", ircfmt_to_ansi(m.text()))
        }
        Payload::ClientMessage(ClientMessage::Mode(m)) if m.modestring().is_some() => {
            let mut comment = m.modestring().expect("is some").to_string();
            if !m.arguments().is_empty() {
                let _ = write!(&mut comment, " {}", m.arguments());
            }
            format!(
                "* {sender} changed the mode for {}: {comment}",
                highlight(m.target().as_str())
            )
        }
        Payload::ClientMessage(_) => format!("[OTHER] Unexpected client message: {msg}"),
        Payload::Reply(_) => format!("[OTHER] Unexpected reply: {msg}"),
    }
}

fn format_msgtext(sender: &str, text: TrailingParam) -> String {
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

fn join_and<I: IntoIterator<Item: AsRef<str>>>(iter: I) -> String {
    let mut items = iter
        .into_iter()
        .map(|it| it.as_ref().to_owned())
        .collect::<Vec<_>>();
    match items.len() {
        0 => String::from("<empty list>"),
        1 => items.pop().expect("Vec is nonempty"),
        2 => format!("{} and {}", items[0], items[1]),
        n => {
            let mut s = String::new();
            for it in &items[..(n - 2)] {
                s.push_str(it);
                s.push_str(", ");
            }
            s.push_str(" and ");
            s.push_str(&items[n - 1]);
            s
        }
    }
}
