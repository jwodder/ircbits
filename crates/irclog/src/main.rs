use anyhow::Context;
use clap::Parser;
use ircnet::client::{
    ClientError, SessionBuilder, SessionParams,
    autoresponders::{CtcpQueryResponder, PingResponder},
    commands::{JoinCommand, JoinOutput, LoginOutput},
};
use irctext::{
    ClientMessage, ClientMessageParts, ClientSource, FinalParam, Message, ParseMessageError,
    Payload, Reply, ReplyParts, Source, TryFromStringError,
    clientmsgs::{Away, Quit},
    ctcp::CtcpParams,
    types::{Channel, ISupportParam},
};
use jiff::{Timestamp, Zoned};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, IsTerminal, Write, stderr};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use tokio::{select, sync::mpsc};
use tracing::Level;
use tracing_subscriber::{
    filter::Targets,
    fmt::{format::Writer, time::FormatTime},
    prelude::*,
};

const MESSAGE_CHANNEL_SIZE: usize = 65535;

#[derive(Clone, Debug, Eq, Parser, PartialEq)]
struct Arguments {
    #[arg(short = 'c', long, default_value = "ircbits.toml")]
    config: PathBuf,

    #[arg(short = 'o', long, default_value = "irclog.jsonl")]
    outfile: PathBuf,

    #[arg(short = 'P', long, default_value = "irc")]
    profile: String,

    #[arg(short = 'R', long)]
    rotate_size: Option<bytesize::ByteSize>,

    #[arg(long)]
    trace: bool,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq)]
struct Profile {
    #[serde(flatten)]
    session_params: SessionParams,

    #[serde(default)]
    irclog: ProgramParams,
}

#[derive(Clone, Debug, Default, serde::Deserialize, Eq, PartialEq)]
struct ProgramParams {
    channels: Vec<Channel>,
    away: Option<FinalParam>,
}

#[tokio::main(worker_threads = 2)]
async fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();
    let loglevel = if args.trace {
        Level::TRACE
    } else {
        Level::INFO
    };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_timer(JiffTimer)
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
    if profile.irclog.channels.is_empty() {
        anyhow::bail!("No channels configured for profile {network:?}");
    }
    let (sender, receiver) = mpsc::channel(MESSAGE_CHANNEL_SIZE);
    let log = EventLogger::new(args.outfile, args.rotate_size.map(|b| b.as_u64()))?;
    let loghandle = tokio::spawn(log_events(log, receiver));
    let r = irc(profile, sender).await;
    let _ = loghandle.await;
    r
}

async fn irc(profile: Profile, sender: mpsc::Sender<Event>) -> anyhow::Result<()> {
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
    sender
        .send(Event::Connected {
            timestamp: Zoned::now(),
            output: login_output,
        })
        .await?;
    if let Some(p) = profile.irclog.away {
        client.send(Away::new(p).into()).await?;
    }
    for chan in profile.irclog.channels {
        tracing::info!("Joining {chan} …");
        let output = client.run(JoinCommand::new(chan.clone())).await?;
        sender
            .send(Event::Joined {
                timestamp: Zoned::now(),
                output,
            })
            .await?;
    }
    loop {
        select! {
            r = client.recv() => {
                match r {
                    Ok(Some(Message {source, payload: Payload::ClientMessage(msg)})) => {
                        sender.send(Event::Message {timestamp: Zoned::now(), source, msg}).await?;
                    }
                    Ok(Some(Message {source, payload: Payload::Reply(reply)})) => {
                        sender.send(Event::Reply {timestamp: Zoned::now(), source, reply}).await?;
                    }
                    Ok(None) => {
                        tracing::info!("Connection closed");
                        sender.send(Event::Disconnected {timestamp: Zoned::now()}).await?;
                        break;
                    }
                    Err(ClientError::Parse(error)) => {
                        sender.send(Event::ParseError {timestamp: Zoned::now(), error}).await?;
                    }
                    Err(e) => {
                        let e = anyhow::Error::new(e);
                        tracing::error!(?e, "Error communicating with server; disconnecting");
                        sender.send(Event::Disconnected {timestamp: Zoned::now()}).await?;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct JiffTimer;

impl FormatTime for JiffTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let now = Zoned::now();
        let ts = now.timestamp();
        let offset = now.offset();
        write!(w, "{}", ts.display_with_offset(offset))
    }
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

async fn log_events(mut log: EventLogger, mut receiver: mpsc::Receiver<Event>) {
    while let Some(ev) = receiver.recv().await {
        if let Err(e) = log.log(ev) {
            tracing::error!(?e, "Failed to write event to log");
            return;
        }
    }
}

#[derive(Debug)]
struct EventLogger {
    filepath: PathBuf,
    file: BufWriter<File>,
    filesize: u64,
    sizelimit: Option<u64>,
}

impl EventLogger {
    fn new(filepath: PathBuf, sizelimit: Option<u64>) -> anyhow::Result<Self> {
        let fp = File::options()
            .create(true)
            .append(true)
            .open(&filepath)
            .context("failed to open output file")?;
        let filesize = fp.metadata().context("failed to stat output file")?.len();
        Ok(EventLogger {
            filepath,
            file: BufWriter::new(fp),
            filesize,
            sizelimit,
        })
    }

    fn log(&mut self, event: Event) -> anyhow::Result<()> {
        let mut line = event.into_json().to_string();
        line.push('\n');
        let linelen = u64::try_from(line.len()).unwrap_or(u64::MAX);
        let mut new_file_size = self.filesize.saturating_add(linelen);
        if let Some(limit) = self.sizelimit
            && new_file_size > limit
        {
            let infix = Timestamp::now().strftime("%Y%m%dT%H%M%SZ").to_string();
            let rotate_path = insert_extension(&self.filepath, &infix);
            tracing::info!("Rotating output file to {} ...", rotate_path.display());
            std::fs::rename(&self.filepath, &rotate_path)
                .context("failed to rotate output file")?;
            let fp = File::options()
                .create(true)
                .append(true)
                .open(&self.filepath)
                .context("failed to reopen output file after rotation")?;
            self.file = BufWriter::new(fp);
            new_file_size = linelen;
        }
        self.file.write_all(line.as_bytes())?;
        self.file.flush()?;
        self.filesize = new_file_size;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Event {
    Connected {
        timestamp: Zoned,
        output: LoginOutput,
    },
    Joined {
        timestamp: Zoned,
        output: JoinOutput,
    },
    Message {
        timestamp: Zoned,
        source: Option<Source>,
        msg: ClientMessage,
    },
    Reply {
        timestamp: Zoned,
        source: Option<Source>,
        reply: Reply,
    },
    ParseError {
        timestamp: Zoned,
        error: TryFromStringError<ParseMessageError>,
    },
    Disconnected {
        timestamp: Zoned,
    },
}

impl Event {
    fn into_json(self) -> Value {
        let mut map = Map::new();
        match self {
            Event::Connected { timestamp, output } => {
                map.insert(String::from("timestamp"), Value::from(fmt_zoned(timestamp)));
                map.insert(String::from("event"), Value::from("connected"));
                output.add_fields(&mut map);
            }
            Event::Joined { timestamp, output } => {
                map.insert(String::from("timestamp"), Value::from(fmt_zoned(timestamp)));
                map.insert(String::from("event"), Value::from("joined"));
                output.add_fields(&mut map);
            }
            Event::Message {
                timestamp,
                source,
                msg,
            } => {
                map.insert(String::from("timestamp"), Value::from(fmt_zoned(timestamp)));
                if let Some(s) = source {
                    s.add_fields(&mut map);
                } else {
                    map.insert(String::from("source"), Value::Null);
                }
                msg.add_fields(&mut map);
            }
            Event::Reply {
                timestamp,
                source,
                reply,
            } => {
                map.insert(String::from("timestamp"), Value::from(fmt_zoned(timestamp)));
                map.insert(String::from("event"), Value::from("reply"));
                if let Some(s) = source {
                    s.add_fields(&mut map);
                } else {
                    map.insert(String::from("source"), Value::Null);
                }
                reply.add_fields(&mut map);
            }
            Event::ParseError { timestamp, error } => {
                map.insert(String::from("timestamp"), Value::from(fmt_zoned(timestamp)));
                map.insert(String::from("event"), Value::from("parse_error"));
                map.insert(String::from("line"), Value::from(error.string));
                map.insert(
                    String::from("error"),
                    Value::from(format!("{:?}", anyhow::Error::new(error.inner))),
                );
            }
            Event::Disconnected { timestamp } => {
                map.insert(String::from("timestamp"), Value::from(fmt_zoned(timestamp)));
                map.insert(String::from("event"), Value::from("disconnected"));
            }
        }
        map.into()
    }
}

trait AddFields: Sized {
    fn add_fields(self, map: &mut Map<String, Value>);
}

impl AddFields for LoginOutput {
    fn add_fields(self, map: &mut Map<String, Value>) {
        let LoginOutput {
            capabilities,
            my_nick,
            server_info,
            isupport,
            luser_stats,
            motd,
            mode,
        } = self;
        if let Some(caps) = capabilities {
            let caplist = caps
                .into_iter()
                .map(|(name, value)| {
                    Map::from_iter([
                        (String::from("name"), Value::from(String::from(name))),
                        (String::from("value"), Value::from(value.map(String::from))),
                    ])
                })
                .collect::<Vec<_>>();
            map.insert(String::from("capabilities"), caplist.into());
        } else {
            map.insert(String::from("capabilities"), Value::Null);
        }
        map.insert(String::from("my_nick"), Value::from(String::from(my_nick)));
        let server = Map::from_iter([
            (String::from("name"), Value::from(server_info.server_name)),
            (String::from("version"), Value::from(server_info.version)),
            (
                String::from("user_modes"),
                Value::from(server_info.user_modes),
            ),
            (
                String::from("channel_modes"),
                Value::from(server_info.channel_modes),
            ),
            (
                String::from("param_channel_modes"),
                Value::from(server_info.param_channel_modes),
            ),
        ]);
        map.insert(String::from("server"), server.into());
        let isupportlist = isupport
            .into_iter()
            .map(|s| match s {
                ISupportParam::Set(key) => (String::from(key), Value::Bool(true)),
                ISupportParam::Unset(key) => (String::from(key), Value::Bool(false)),
                ISupportParam::Eq(key, value) => {
                    (String::from(key), Value::from(String::from(value)))
                }
            })
            .collect::<Map<_, _>>();
        map.insert(String::from("isupport"), isupportlist.into());
        let lusers = Map::from_iter([
            (
                String::from("operators"),
                Value::from(luser_stats.operators),
            ),
            (
                String::from("unknown_connections"),
                Value::from(luser_stats.unknown_connections),
            ),
            (String::from("channels"), Value::from(luser_stats.channels)),
            (
                String::from("local_clients"),
                Value::from(luser_stats.local_clients),
            ),
            (
                String::from("max_local_clients"),
                Value::from(luser_stats.max_local_clients),
            ),
            (
                String::from("global_clients"),
                Value::from(luser_stats.global_clients),
            ),
            (
                String::from("max_global_clients"),
                Value::from(luser_stats.max_global_clients),
            ),
        ]);
        map.insert(String::from("lusers"), lusers.into());
        map.insert(String::from("motd"), Value::from(motd));
        map.insert(String::from("mode"), Value::from(mode.map(String::from)));
    }
}

impl AddFields for JoinOutput {
    fn add_fields(self, map: &mut Map<String, Value>) {
        let JoinOutput {
            channel,
            topic,
            topic_set_by,
            topic_set_at,
            channel_status,
            users,
        } = self;
        map.insert(String::from("channel"), Value::from(String::from(channel)));
        map.insert(String::from("topic"), Value::from(topic));
        if let Some(clisrc) = topic_set_by {
            let mut srcmap = Map::new();
            clisrc.add_fields(&mut srcmap);
            map.insert(String::from("topic_set_by"), srcmap.into());
        } else {
            map.insert(String::from("topic_set_by"), Value::Null);
        }
        map.insert(
            String::from("topic_set_at"),
            if let Some(ts) = topic_set_at {
                if let Some(s) = fmt_unix_timestamp(ts) {
                    Value::from(s)
                } else {
                    Value::from(topic_set_at)
                }
            } else {
                Value::Null
            },
        );
        map.insert(
            String::from("channel_status"),
            Value::from(format!("{channel_status:?}")),
        );
        let users = users
            .into_iter()
            .map(|(memship, nick)| {
                Map::from_iter([
                    (String::from("nickname"), Value::from(String::from(nick))),
                    (
                        String::from("membership"),
                        Value::from(memship.map(|m| format!("{m:?}"))),
                    ),
                ])
            })
            .collect::<Vec<_>>();
        map.insert(String::from("users"), Value::from(users));
    }
}

impl AddFields for Source {
    fn add_fields(self, map: &mut Map<String, Value>) {
        let mut srcmap = Map::new();
        match self {
            Source::Server(host) => {
                srcmap.insert(String::from("host"), Value::from(host.to_string()));
            }
            Source::Client(clisrc) => clisrc.add_fields(&mut srcmap),
        }
        map.insert(String::from("source"), srcmap.into());
    }
}

impl AddFields for ClientSource {
    fn add_fields(self, map: &mut Map<String, Value>) {
        let ClientSource {
            nickname,
            user,
            host,
        } = self;
        map.insert(
            String::from("nickname"),
            Value::from(String::from(nickname)),
        );
        map.insert(String::from("user"), Value::from(user.map(String::from)));
        map.insert(String::from("host"), Value::from(host));
    }
}

impl AddFields for ClientMessage {
    // Includes "event" field
    fn add_fields(self, map: &mut Map<String, Value>) {
        match self {
            ClientMessage::Admin(msg) => {
                map.insert(String::from("event"), Value::from("admin"));
                map.insert(
                    String::from("target"),
                    Value::from(msg.into_target().map(String::from)),
                );
            }
            ClientMessage::Authenticate(msg) => {
                map.insert(String::from("event"), Value::from("authenticate"));
                map.insert(
                    String::from("parameter"),
                    Value::from(String::from(msg.into_parameter())),
                );
            }
            ClientMessage::Away(msg) => {
                map.insert(String::from("event"), Value::from("away"));
                map.insert(
                    String::from("text"),
                    Value::from(msg.into_text().map(String::from)),
                );
            }
            ClientMessage::Cap(msg) => {
                let (_, params) = msg.into_parts();
                map.insert(String::from("event"), Value::from("cap"));
                map.insert(
                    String::from("parameters"),
                    Value::from(params.into_iter().map(String::from).collect::<Vec<_>>()),
                );
            }
            ClientMessage::Connect(msg) => {
                map.insert(String::from("event"), Value::from("connect"));
                map.insert(
                    String::from("target_server"),
                    Value::from(msg.target_server().to_string()),
                );
                map.insert(String::from("port"), Value::from(msg.port()));
                map.insert(
                    String::from("remote_server"),
                    Value::from(msg.remote_server().map(ToString::to_string)),
                );
            }
            ClientMessage::Error(msg) => {
                map.insert(String::from("event"), Value::from("error"));
                map.insert(
                    String::from("reason"),
                    Value::from(String::from(msg.into_reason())),
                );
            }
            ClientMessage::Help(msg) => {
                map.insert(String::from("event"), Value::from("help"));
                map.insert(
                    String::from("subject"),
                    Value::from(msg.into_subject().map(String::from)),
                );
            }
            ClientMessage::Info(_) => {
                map.insert(String::from("event"), Value::from("info"));
            }
            ClientMessage::Invite(msg) => {
                map.insert(String::from("event"), Value::from("invite"));
                map.insert(
                    String::from("nickname"),
                    Value::from(msg.nickname().to_string()),
                );
                map.insert(
                    String::from("channel"),
                    Value::from(msg.channel().to_string()),
                );
            }
            ClientMessage::Join(msg) => {
                if msg.is_zero() {
                    map.insert(String::from("event"), Value::from("join0"));
                } else {
                    map.insert(String::from("event"), Value::from("join"));
                    map.insert(
                        String::from("channels"),
                        Value::from(
                            msg.channels()
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>(),
                        ),
                    );
                    // Don't bother with the keys field
                }
            }
            ClientMessage::Kick(msg) => {
                map.insert(String::from("event"), Value::from("kick"));
                map.insert(
                    String::from("channel"),
                    Value::from(msg.channel().to_string()),
                );
                map.insert(
                    String::from("users"),
                    Value::from(
                        msg.users()
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>(),
                    ),
                );
                map.insert(
                    String::from("comment"),
                    Value::from(msg.comment().map(ToString::to_string)),
                );
            }
            ClientMessage::Kill(msg) => {
                map.insert(String::from("event"), Value::from("kill"));
                map.insert(
                    String::from("nickname"),
                    Value::from(msg.nickname().to_string()),
                );
                map.insert(
                    String::from("comment"),
                    Value::from(msg.comment().to_string()),
                );
            }
            ClientMessage::Links(_) => {
                map.insert(String::from("event"), Value::from("links"));
            }
            ClientMessage::List(msg) => {
                map.insert(String::from("event"), Value::from("list"));
                map.insert(
                    String::from("channels"),
                    Value::from(
                        msg.channels()
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>(),
                    ),
                );
                map.insert(
                    String::from("elistconds"),
                    Value::from(
                        msg.elistconds()
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>(),
                    ),
                );
            }
            ClientMessage::Lusers(_) => {
                map.insert(String::from("event"), Value::from("lusers"));
            }
            ClientMessage::Mode(msg) => {
                map.insert(String::from("event"), Value::from("mode"));
                map.insert(
                    String::from("target"),
                    Value::from(msg.target().to_string()),
                );
                map.insert(
                    String::from("modestring"),
                    Value::from(msg.modestring().map(ToString::to_string)),
                );
                map.insert(
                    String::from("arguments"),
                    Value::from(
                        msg.arguments()
                            .iter()
                            .map(|a| a.to_string())
                            .collect::<Vec<_>>(),
                    ),
                );
            }
            ClientMessage::Motd(msg) => {
                map.insert(String::from("event"), Value::from("motd"));
                map.insert(
                    String::from("target"),
                    Value::from(msg.into_target().map(String::from)),
                );
            }
            ClientMessage::Names(msg) => {
                map.insert(String::from("event"), Value::from("motd"));
                map.insert(
                    String::from("channels"),
                    Value::from(
                        msg.into_channels()
                            .into_iter()
                            .map(String::from)
                            .collect::<Vec<_>>(),
                    ),
                );
            }
            ClientMessage::Nick(msg) => {
                map.insert(String::from("event"), Value::from("nick"));
                map.insert(
                    String::from("nickname"),
                    Value::from(String::from(msg.into_nickname())),
                );
            }
            ClientMessage::Notice(msg) => {
                map.insert(String::from("event"), Value::from("notice"));
                map.insert(
                    String::from("targets"),
                    Value::from(
                        msg.targets()
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>(),
                    ),
                );
                map.insert(String::from("text"), Value::from(msg.text().to_string()));
            }
            ClientMessage::Oper(msg) => {
                map.insert(String::from("event"), Value::from("oper"));
                map.insert(String::from("name"), Value::from(msg.name().to_string()));
                map.insert(
                    String::from("password"),
                    Value::from(msg.password().to_string()),
                );
            }
            ClientMessage::Part(msg) => {
                map.insert(String::from("event"), Value::from("part"));
                map.insert(
                    String::from("channels"),
                    Value::from(
                        msg.channels()
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>(),
                    ),
                );
                map.insert(
                    String::from("reason"),
                    Value::from(msg.reason().map(ToString::to_string)),
                );
            }
            ClientMessage::Pass(msg) => {
                map.insert(String::from("event"), Value::from("pass"));
                map.insert(
                    String::from("password"),
                    Value::from(String::from(msg.into_password())),
                );
            }
            ClientMessage::Ping(msg) => {
                map.insert(String::from("event"), Value::from("pong"));
                map.insert(
                    String::from("token"),
                    Value::from(String::from(msg.into_token())),
                );
            }
            ClientMessage::Pong(msg) => {
                map.insert(String::from("event"), Value::from("ping"));
                map.insert(
                    String::from("token"),
                    Value::from(String::from(msg.into_token())),
                );
            }
            ClientMessage::PrivMsg(msg) => {
                map.insert(String::from("event"), Value::from("privmsg"));
                map.insert(
                    String::from("targets"),
                    Value::from(
                        msg.targets()
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>(),
                    ),
                );
                map.insert(String::from("text"), Value::from(msg.text().to_string()));
            }
            ClientMessage::Quit(msg) => {
                map.insert(String::from("event"), Value::from("quit"));
                map.insert(
                    String::from("reason"),
                    Value::from(msg.into_reason().map(String::from)),
                );
            }
            ClientMessage::Rehash(_) => {
                map.insert(String::from("event"), Value::from("rehash"));
            }
            ClientMessage::Restart(_) => {
                map.insert(String::from("event"), Value::from("restart"));
            }
            ClientMessage::Squit(msg) => {
                map.insert(String::from("event"), Value::from("squit"));
                map.insert(
                    String::from("server"),
                    Value::from(msg.server().to_string()),
                );
                map.insert(
                    String::from("comment"),
                    Value::from(msg.comment().to_string()),
                );
            }
            ClientMessage::Stats(msg) => {
                map.insert(String::from("event"), Value::from("stats"));
                map.insert(String::from("query"), Value::from(msg.query().to_string()));
                map.insert(
                    String::from("server"),
                    Value::from(msg.server().map(ToString::to_string)),
                );
            }
            ClientMessage::Time(msg) => {
                map.insert(String::from("event"), Value::from("time"));
                map.insert(
                    String::from("server"),
                    Value::from(msg.into_server().map(String::from)),
                );
            }
            ClientMessage::Topic(msg) => {
                map.insert(String::from("event"), Value::from("topic"));
                map.insert(
                    String::from("channel"),
                    Value::from(msg.channel().to_string()),
                );
                map.insert(
                    String::from("topic"),
                    Value::from(msg.topic().map(ToString::to_string)),
                );
            }
            ClientMessage::User(msg) => {
                map.insert(String::from("event"), Value::from("user"));
                map.insert(
                    String::from("username"),
                    Value::from(msg.username().to_string()),
                );
                map.insert(
                    String::from("realname"),
                    Value::from(msg.realname().to_string()),
                );
            }
            ClientMessage::UserHost(msg) => {
                map.insert(String::from("event"), Value::from("userhost"));
                map.insert(
                    String::from("nicknames"),
                    Value::from(
                        msg.into_nicknames()
                            .into_iter()
                            .map(String::from)
                            .collect::<Vec<_>>(),
                    ),
                );
            }
            ClientMessage::Version(msg) => {
                map.insert(String::from("event"), Value::from("version"));
                map.insert(
                    String::from("target"),
                    Value::from(msg.into_target().map(String::from)),
                );
            }
            ClientMessage::Wallops(msg) => {
                map.insert(String::from("event"), Value::from("wallops"));
                map.insert(
                    String::from("text"),
                    Value::from(String::from(msg.into_text())),
                );
            }
            ClientMessage::Who(msg) => {
                map.insert(String::from("event"), Value::from("who"));
                map.insert(
                    String::from("mask"),
                    Value::from(String::from(msg.into_mask())),
                );
            }
            ClientMessage::WhoIs(msg) => {
                map.insert(String::from("event"), Value::from("whois"));
                map.insert(
                    String::from("target"),
                    Value::from(msg.target().map(ToString::to_string)),
                );
                map.insert(
                    String::from("nickname"),
                    Value::from(msg.nickname().to_string()),
                );
            }
            ClientMessage::WhoWas(msg) => {
                map.insert(String::from("event"), Value::from("whowas"));
                map.insert(
                    String::from("nickname"),
                    Value::from(msg.nickname().to_string()),
                );
                map.insert(
                    String::from("count"),
                    Value::from(msg.count().map(NonZeroUsize::get)),
                );
            }
        }
    }
}

impl AddFields for Reply {
    fn add_fields(self, map: &mut Map<String, Value>) {
        let (code, params) = self.into_parts();
        map.insert(String::from("code"), Value::from(code));
        map.insert(
            String::from("parameters"),
            Value::from(params.into_iter().map(String::from).collect::<Vec<_>>()),
        );
    }
}

fn fmt_zoned(z: Zoned) -> String {
    let ts = z.timestamp();
    let offset = z.offset();
    ts.display_with_offset(offset).to_string()
}

fn fmt_unix_timestamp(ts: u64) -> Option<String> {
    let its = i64::try_from(ts).ok()?;
    let jts = Timestamp::from_second(its).ok()?;
    Some(jts.to_string())
}

fn insert_extension(path: &Path, infix: &str) -> PathBuf {
    if let Some(ext) = path.extension() {
        path.with_extension(infix).with_added_extension(ext)
    } else {
        path.with_extension(infix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod insert_extension {
        use super::*;

        #[test]
        fn basic() {
            let p = insert_extension(Path::new("foo.txt"), "123");
            assert_eq!(p, Path::new("foo.123.txt"));
        }

        #[test]
        fn no_ext() {
            let p = insert_extension(Path::new("foo"), "123");
            assert_eq!(p, Path::new("foo.123"));
        }

        #[test]
        fn trailing_dot() {
            let p = insert_extension(Path::new("foo."), "123");
            assert_eq!(p, Path::new("foo.123"));
        }
    }
}
