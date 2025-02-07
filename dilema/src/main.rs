use std::{thread, time::Duration};

use anyhow::Context;
use dilema::Dilema;
use smithay::reexports::calloop::EventLoop;

fn main() -> anyhow::Result<()> {
    let _guard = setup_tracing();
    let mut event_loop = EventLoop::<Dilema>::try_new().context("failed to setup event loop")?;
    let mut dilema = Dilema::setup(&mut event_loop)?;

    // prevent deadlock
    // let signal = event_loop.get_signal();
    // thread::spawn(move||{
    //     thread::sleep(Duration::from_secs(10));
    //     tracing::debug!("stoping loop...");
    //     signal.stop();
    //     tracing::debug!("loop stoped");
    // });

    tracing::debug!("event loop running...");
    // Ok(event_loop.run(Duration::from_secs(4), &mut dilema, Dilema::refresh)?)
    let mut counter = 0;

    while counter < 10 {
        tracing::debug!("looped");
        let result = event_loop.dispatch(Some(Duration::from_millis(16)), &mut dilema);
        tracing::debug!("refresh: {result:?}");
        dilema.refresh();
        counter += 1;
        std::thread::sleep(Duration::from_millis(500));
    }

    drop(event_loop);
    drop(dilema.session);

    tracing::info!("exiting");
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

