use smithay::reexports::calloop::EventLoop;
use vice::Vice;

fn main() {
    let _guard = setup_tracing();
    let mut event_loop = EventLoop::try_new().unwrap();
    let mut vice = Vice::setup(&mut event_loop).unwrap();

    event_loop.run(None, &mut vice, |_|{}).unwrap();
    println!("Hello, world!");
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

