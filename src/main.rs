// this is binary crate, compatibility is not a problem
#![allow(refining_impl_trait)]
use std::process::ExitCode;
use todex::sys::listener::{Listener, SocketPath};
use todex::sys::sigfd::Sigfd;
use todex::rt::poller::Poller;

use buffer::BufferPool;
use client::Clients;
use compositor::Compositor;
use service::clients::ClientService;
use service::listener::ListenerService;
use error::FatalError;

mod buffer;
mod seat;
mod client;

mod compositor;

mod service;

mod log;
mod error;

// TODO: destructor trait
// TODO: global object trait, for version checking
// TODO: returning interface specific error

const SOCKET_PATH: SocketPath = SocketPath::new(c"/tmp/wayland-2");

const STATIC_FLAG: u64 = i64::MIN as u64;
const LISTENER_KEY: u64 = STATIC_FLAG;
const SIGFD_KEY: u64 = STATIC_FLAG | 1;

fn main() -> ExitCode {
    let _guard = log::init();
    <_>::from(event_loop().is_err() as u8)
}

pub fn event_loop() -> Result<(), FatalError> {
    // ===== sys =====
    let listener = Listener::new(SOCKET_PATH)?;
    let sigfd = Sigfd::new()?;
    let mut poll = Poller::new()?;

    // ===== states =====
    let mut clients = Clients::new();
    let mut buffer = BufferPool::new();
    let mut compositor = Compositor::new()?;

    // ===== services =====
    let mut listener_service = ListenerService::new(&listener);
    let mut client_service = ClientService::new();

    poll.add(LISTENER_KEY, &listener);
    poll.add(SIGFD_KEY, &sigfd);

    loop {
        let Some((key, interest)) = poll.next_event() else {
            log::debug!(target: "polling", "blocking");
            log::flush();
            poll.wait(None);
            continue;
        };
        if key & STATIC_FLAG != 0 {
            match key {
                SIGFD_KEY => {
                    log::info!("{} signal received", sigfd.read());
                    break;
                }
                LISTENER_KEY => listener_service.serve(&poll, &mut clients),
                _ => {}
            }
        } else {
            client_service.serve(
                key,
                interest,
                &poll,
                &mut buffer,
                &mut compositor,
                &mut clients,
            );
        }
    }

    Ok(())
}
