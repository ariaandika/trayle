use anyhow::Context;
use trayle::Trayle;
use smithay::reexports::calloop::EventLoop;

fn main() -> anyhow::Result<()> {
    let _guard = setup_tracing();
    let mut event_loop = EventLoop::<Trayle>::try_new().context("failed to setup event loop")?;
    let mut trayle = Trayle::setup(&mut event_loop)?;
    event_loop.run(None, &mut trayle, Trayle::refresh).unwrap();
    Ok(())
}

fn setup_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    use tracing_appender::{rolling::never, non_blocking};
    std::fs::remove_file(".log").ok();
    let (log, guard) = non_blocking(never(".", ".log"));
    tracing_subscriber::fmt()
        .with_writer(log)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    guard
}

