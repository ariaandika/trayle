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
use epoll::{Epoll, EpollBuf};
use error::Result;
use id::{IdManager, Id};
use listener::{Listener, SocketPath};
use sigfd::Sigfd;

mod macros;
mod error;
mod ptr;
mod buffer;
mod conn;
mod listener;
mod epoll;
mod sigfd;
mod id;
mod wayland;

const SOCKET_PATH: SocketPath = SocketPath::new(c"/tmp/wayland-2");

const LISTENER_ID: Id = IdManager::generate_static(0);
const SIGFD_ID: Id = IdManager::generate_static(1);

const MAX_FD: u32 = 32;
const MAX_FD_SIZE: u32 = MAX_FD * size_of::<i32>() as u32;

fn main() -> error::Terminate {
    event_loop().into()
}

fn event_loop() -> Result<()> {
    // ===== setup =====

    let listener = Listener::new(SOCKET_PATH)?;
    let sigfd = Sigfd::new()?;
    let mut epoll = Epoll::new()?;
    let mut id_manager = IdManager::new();

    epoll.add_read_interest(LISTENER_ID, &listener)?;
    epoll.add_read_interest(SIGFD_ID, &sigfd)?;

    let mut epoll_buf = EpollBuf::new();
    let mut streams = Vec::with_capacity(8);
    let mut read_buffer = Buffer::with_capacity(1024);
    let mut fds_buffer = Buffer::with_capacity(MAX_FD_SIZE);

    // ===== event loop =====

    loop {
        let Some((key, interest)) = epoll.next_event(&epoll_buf) else {
            eprintln!("[EPOLL] blocking");
            epoll.wait(&mut epoll_buf)?;
            eprintln!("[EPOLL] complete");
            continue;
        };
        let id = Id::from_u64(key);
        if id.is_static() {
            match id {
                LISTENER_ID => {
                    while let Ready(result) = listener.poll_accept() {
                        let stream = match result {
                            Ok(ok) => ok,
                            Err(err) => {
                                eprintln!("[CLIENT] cannot connect: {err}");
                                continue;
                            }
                        };
                        let id_new = id_manager.generate_dynamic(streams.len() as u32);
                        if let Err(err) = epoll.add_read_interest(id_new, &stream) {
                            eprintln!("[CLIENT] cannot add to epoll: {err}");
                            continue;
                        }
                        streams.push(stream);
                        println!("[CLIENT] connected");
                    }
                }
                SIGFD_ID => {
                    sigfd.read()?;
                    break;
                }
                _ => eprintln!("[EPOLL] invalid static key")
            }
        } else {
            let idx = id.value() as usize;
            if interest.is_close() {
                let stream = streams.swap_remove(idx);
                if let Err(err) = epoll.remove_interest(&stream) {
                    eprintln!("cannot remove epoll interest: {err}");
                }
                println!("[CLIENT]: close");
            } else {
                let stream = &mut streams[idx];
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
    }

    Ok(())
}
