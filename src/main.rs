#![allow(refining_impl_trait, clippy::module_inception)]
use std::process::ExitCode;
use todex::sys::sigfd::Sigfd;
use todex::log;

use error::FatalError;
use client::Gateway;
use compositor::Compositor;
use input::Input;
use poller::{EventKind, Poller};

// ===== mods =====

mod handle;
mod error;

mod shm;
mod surface;
mod buffer;

mod seat;
mod client;
mod compositor;
mod input;

mod poller;

// ===== impls =====

fn main() -> ExitCode {
    let _guard = log::init();
    <_>::from(event_loop().is_err() as u8)
}

fn event_loop() -> Result<(), FatalError> {
    let sigfd = Sigfd::new()?;

    let mut input = Input::new()?;
    let mut compositor = Compositor::new()?;
    let mut gateway = Gateway::new()?;

    let mut poll = Poller::new()?;
    poll.add_source(&input);
    poll.add_source(&gateway);
    poll.add_source(&sigfd);

    loop {
        let Some((event, kind)) = poll.next_event() else {
            log::debug!(target: "polling", "blocking");
            log::flush();
            poll.wait(None);
            continue;
        };
        match kind {
            EventKind::Client => gateway.dispatch_io(event, &poll, &mut compositor),
            EventKind::Input => input.dispatch().for_each(drop),
            EventKind::Gateway => gateway.dispatch(&poll),
            EventKind::Sigfd => {
                log::info!("{} signal received", sigfd.read());
                break Ok(());
            }
        }
    }
}
