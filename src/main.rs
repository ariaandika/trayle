#![allow(refining_impl_trait, clippy::module_inception)]
use std::process::ExitCode;
use todex::sys::epoll::Epoll;
use todex::sys::listener::{Listener, SocketPath};
use todex::sys::sigfd::Sigfd;
use todex::log;

use buffer::BufferPool;
use client::{Clients, Gateway};
use compositor::Compositor;
use poller::Poller;
use error::FatalError;

mod handle;
mod shm;
mod surface;
mod buffer;
mod seat;
mod client;
mod compositor;
mod poller;
mod error;

const SOCKET_PATH: SocketPath = SocketPath::new(c"/tmp/wayland-2");
const MSB: u64 = i64::MIN as u64;

const LISTENER_KEY: u64 = MSB;
const SIGFD_KEY: u64 = MSB | 2;

const EVENT_BUF: usize = 128;

fn main() -> ExitCode {
    let _guard = log::init();
    <_>::from(event_loop().is_err() as u8)
}

pub fn event_loop() -> Result<(), FatalError> {
    // ===== sys =====
    let listener = Listener::new(SOCKET_PATH)?;
    let sigfd = Sigfd::new()?;
    let epoll = Epoll::new()?;

    // ===== components =====
    let mut clients = Clients::new();
    let mut buffer = BufferPool::new();

    // ===== domain =====
    let mut compositor = Compositor::new()?;

    // ===== reactor =====
    let mut gateway = Gateway::new(&epoll, &listener);

    // ===== poller =====
    let mut poll = Poller::new(EVENT_BUF, &epoll);

    epoll.add(LISTENER_KEY, &listener);
    epoll.add(SIGFD_KEY, &sigfd);

    loop {
        let Some(event) = poll.next_event() else {
            log::debug!(target: "polling", "blocking");
            log::flush();
            poll.wait(None);
            continue;
        };
        if event.key & MSB == MSB {
            match event.key {
                LISTENER_KEY => gateway.dispatch_listener(&mut clients),
                SIGFD_KEY => {
                    log::info!("{} signal received", sigfd.read());
                    break Ok(());
                }
                key => log::error!("unknown key from epoll: {key}")
            }
        } else {
            gateway.dispatch_io(event, &mut buffer, &mut clients, &mut compositor);
        }
    }
}
