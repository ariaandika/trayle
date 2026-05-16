//! Wayland server implementation.
//!
//! # Shared
//!
//! - [`macros`] utility macros
//! - [`error`] error types and util
//!
//! # Memory Management
//!
//! - [`buffer`] bytes buffer and cursor
//!
//! # Network
//!
//! - [`conn`] client socket connection
//! - [`listener`] socket listener
//!
//! # System
//!
//! - [`epoll`] epoll based event loop
//! - [`sigfd`] handle process signal
//!
//! # Application
//!
//! - [`wayland`] contains all wayland logic
use std::task::Poll::*;

use buffer::Buffer;
use client::Client;
use clients::Clients;
use epoll::Epoll;
use error::Result;
use listener::{Listener, SocketPath};
use sigfd::Sigfd;

// ===== shared ====
mod macros;
mod error;
// ===== os ========
mod conn;
mod listener;
mod epoll;
mod sigfd;
// ===== alloc =====
mod ptr;
mod buffer;
// ===== app =======
mod client;
mod clients;
mod wayland;

const SOCKET_PATH: SocketPath = SocketPath::new(c"/tmp/wayland-2");

const STATIC_ID_MASK: u64 = i64::MIN as u64;
const LISTENER_ID: u64 = STATIC_ID_MASK | 1;
const SIGFD_ID: u64 = STATIC_ID_MASK | 2;

const MAX_EPOLL_EVENT: usize = 128;
const MAX_FD: u32 = 32;
const MAX_FD_SIZE: u32 = MAX_FD * size_of::<i32>() as u32;

fn main() -> error::Terminate {
    event_loop().into()
}

fn event_loop() -> Result<()> {

    // ===== os =====
    let listener = Listener::new(SOCKET_PATH)?;
    let sigfd = Sigfd::new()?;
    let epoll = Epoll::new()?;

    epoll.add_read(LISTENER_ID, &listener)?;
    epoll.add_read(SIGFD_ID, &sigfd)?;

    // ===== alloc =====
    let mut events_i = 0;
    let mut events = Vec::with_capacity(MAX_EPOLL_EVENT);
    let mut read_buffer = Buffer::with_capacity(1024);
    let mut fds_buffer = Buffer::with_capacity(MAX_FD_SIZE);

    // ===== app =====
    let mut clients = Clients::with_capacity(8);

    // ===== event loop =====

    loop {
        let Some(event) = events.get(events_i) else {
            eprintln!("[EPOLL] blocking");
            events_i = 0;
            events.clear();
            let n = epoll.wait(events.spare_capacity_mut(), None)?;
            unsafe { events.set_len(n) };
            eprintln!("[EPOLL] complete");
            continue;
        };
        events_i += 1;

        let id = event.key();
        let interest = event.interest();

        if id & STATIC_ID_MASK == STATIC_ID_MASK {
            match id {
                LISTENER_ID => loop {
                    let conn = match listener.poll_accept() {
                        Ready(Ok(ok)) => ok,
                        Ready(Err(err)) => {
                            eprintln!("[CLIENT] cannot connect: {err}");
                            break;
                        },
                        Pending => break,
                    };
                    let new_id = clients.peek_id();
                    if let Err(err) = epoll.add_read(new_id, &conn) {
                        eprintln!("[CLIENT] cannot add to epoll: {err}");
                        continue;
                    }
                    clients.insert(Client::new(new_id, conn));
                    println!("[CLIENT] connected {:?}", Clients::destruct_id(new_id));
                }
                SIGFD_ID => {
                    match sigfd.read() {
                        Ok(Some(sig)) => eprintln!("[SIGFD] {sig} signal received"),
                        Ok(None) => eprintln!("[SIGFD] unrecognized signal"),
                        Err(err) => eprintln!("[SIGFD] error: {err}"),
                    }
                    break;
                }
                _ => eprintln!("[EPOLL] invalid static key")
            }
            continue;
        }

        if interest.is_close() {
            let Some(client) = clients.remove(id) else {
                eprintln!("[CLIENT] failed to remove client, invalid id from epoll");
                continue;
            };
            if let Err(err) = epoll.remove(&client) {
                eprintln!("cannot remove epoll interest: {err}");
            }
            println!("[CLIENT] close");
            continue;
        }

        let Some(client) = clients.get_mut(id) else {
            eprintln!("[CLIENT] failed to get client, invalid id from epoll");
            continue;
        };
        loop {
            match client.poll_read(&mut read_buffer, &mut fds_buffer) {
                Ready(Ok(())) => {}
                Ready(Err(err)) => {
                    eprintln!("[CLIENT] failed to read: {err}");
                    break;
                }
                Pending => break,
            }

            while let Some((header, rest)) = wayland::split_header(&read_buffer) {
                let (id, op, len) = header;
                let Some((body, _rest)) = rest.split_at_checked(len as usize) else {
                    break;
                };
                dbg!((id, op, len));
                dbg!(body.len());
                read_buffer.advance((8 + len) as u32);
            }
        }
    }

    Ok(())
}
