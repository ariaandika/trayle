#![allow(refining_impl_trait, clippy::module_inception)]
use std::process::ExitCode;
use todex::sys::epoll::Epoll;
use todex::sys::listener::{Listener, SocketPath};
use todex::sys::sigfd::Sigfd;
use todex::poller::Poller;
use todex::log;

use buffer::BufferPool;
use client::{Clients, ClientReactor};
use compositor::Compositor;
use error::FatalError;

mod handle;
mod shm;
mod surface;
mod buffer;
mod seat;
mod client;
mod compositor;
mod error;

const SOCKET_PATH: SocketPath = SocketPath::new(c"/tmp/wayland-2");
const MSB: u64 = i64::MIN as u64;

const LISTENER_KEY: u64 = MSB;
const SIGFD_KEY: u64 = MSB | 1;

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
    let mut client_reactor = ClientReactor::new(&epoll, &listener);

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
                SIGFD_KEY => {
                    log::info!("{} signal received", sigfd.read());
                    break Ok(());
                }
                LISTENER_KEY => client_reactor.handle_listener(&mut clients),
                _ => {}
            }
        } else {
            client_reactor.handle_socket(event, &mut buffer, &mut clients, &mut compositor);
        }
    }
}
