use std::io::{IsTerminal, stderr};
use tokio::select;
use tracing::Level;
use tracing_subscriber::{
    filter::Targets,
    fmt::{format::Writer, time::FormatTime},
    prelude::*,
};

pub fn init_logging(cratename: &str, loglevel: Level) {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_timer(JiffTimer)
                .with_ansi(stderr().is_terminal())
                .with_writer(stderr),
        )
        .with(
            Targets::new()
                .with_target(cratename, loglevel)
                .with_target("ircnet", loglevel)
                .with_default(Level::INFO),
        )
        .init();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct JiffTimer;

impl FormatTime for JiffTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let now = jiff::Zoned::now();
        let ts = now.timestamp();
        let offset = now.offset();
        write!(w, "{}", ts.display_with_offset(offset))
    }
}

pub async fn run_until_stopped<Fut: Future>(fut: Fut) -> Option<Fut::Output> {
    select! {
        r = fut => Some(r),
        () = recv_stop_signal() => None,
    }
}

#[cfg(unix)]
pub async fn recv_stop_signal() -> () {
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
pub async fn recv_stop_signal() -> () {
    let _ = tokio::signal::ctrl_c().await;
}
