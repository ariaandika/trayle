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
//! - [`id`] client identifiers
//! - [`wayland`] contains all wayland logic
use std::task::Poll::*;

use buffer::Buffer;
use client::Client;
use epoll::{Epoll, EpollBuf};
use error::Result;
use id::{Id, IdManager};
use listener::{Listener, SocketPath};
use sigfd::Sigfd;

// ===== os =====

mod conn;
mod listener;
mod epoll;
mod sigfd;

// ===== alloc =====

mod ptr;
mod buffer;

// ===== type =====

mod macros;
mod error;
mod id;
mod client;
mod wayland;

const SOCKET_PATH: SocketPath = SocketPath::new(c"/tmp/wayland-2");

const LISTENER_ID: Id = IdManager::generate_static(0);
const SIGFD_ID: Id = IdManager::generate_static(1);

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

    epoll.add_read_interest(LISTENER_ID, &listener)?;
    epoll.add_read_interest(SIGFD_ID, &sigfd)?;

    // ===== alloc =====

    let mut id_manager = IdManager::new();
    let mut epoll_buf = EpollBuf::new();
    let mut conns = Vec::with_capacity(8);
    let mut read_buffer = Buffer::with_capacity(1024);
    let mut fds_buffer = Buffer::with_capacity(MAX_FD_SIZE);

    // ===== event loop =====

    loop {
        let Some((key, interest)) = epoll.next_event(&mut epoll_buf) else {
            eprintln!("[EPOLL] blocking");
            epoll.wait(&mut epoll_buf)?;
            eprintln!("[EPOLL] complete");
            continue;
        };
        let id = Id::from_u64(key);
        if id.is_static() {
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
                    let new_id = id_manager.generate_dynamic(conns.len() as u32);
                    if let Err(err) = epoll.add_read_interest(new_id, &conn) {
                        eprintln!("[CLIENT] cannot add to epoll: {err}");
                        continue;
                    }
                    conns.push(Client::new(new_id, conn));
                    println!("[CLIENT] connected");
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
        } else {
            let idx = id.value() as usize;
            if interest.is_close() {
                let stream = conns.swap_remove(idx);
                if let Err(err) = epoll.remove_interest(&stream) {
                    eprintln!("cannot remove epoll interest: {err}");
                }
                println!("[CLIENT]: close");
                continue;
            }
            let stream = &mut conns[idx];
            loop {
                match stream.poll_read(&mut read_buffer, &mut fds_buffer) {
                    Ready(Ok(())) => {}
                    Ready(Err(err)) => {
                        eprintln!("[CLIENT] failed to read: {err}");
                        break;
                    }
                    Pending => break,
                }
                println!("[CLIENT]: {:?}", str::from_utf8(&read_buffer));
                read_buffer.clear();
            }
        }
    }

    Ok(())
}
