//! Wayland server implementation.
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
//! # Protocol
//!
//! - [`wayland`] contains all wayland logic
//!
//! # Application
//!
//! - [`handler`] main logic for handling request
use std::process::ExitCode;
use std::task::Poll::*;

use sys::listener::{Listener, SocketPath};
use sys::sigfd::Sigfd;
use collections::buffer::Buffer;
use wayland::Frame;
use compositor::clients::{ClientId, ClientMut, Clients};
use compositor::seat::Seat;
use rt::event::EventSources;

mod sys;
mod alloc;
mod collections;
mod wayland;
mod compositor;
mod rt;
mod handler;
mod log;

const SOCKET_PATH: SocketPath = SocketPath::new(c"/tmp/wayland-2");

const STATIC_ID_MASK: u64 = i64::MIN as u64;
const LISTENER_ID: u64 = STATIC_ID_MASK | 1;
const SIGFD_ID: u64 = STATIC_ID_MASK | 2;

pub struct FatalError;

impl<E: std::fmt::Display> From<E> for FatalError {
    fn from(value: E) -> Self {
        log::error!(setup, "{value}");
        Self
    }
}

pub struct Compositor {
    seat: Seat,
}

fn main() -> ExitCode {
    let _guard = log::init();
    <_>::from(event_loop().is_err() as u8)
}

fn event_loop() -> Result<(), FatalError> {
    let seat = Seat::new()?;
    let listener = Listener::new(SOCKET_PATH)?;
    let sigfd = Sigfd::new()?;

    let mut read_buf = Buffer::new();
    let mut write_buf = Buffer::new();

    let mut clients = Clients::new();
    let mut compositor = Compositor { seat };

    let mut events = EventSources::new()?;

    events.add(LISTENER_ID, &listener);
    events.add(SIGFD_ID, &sigfd);

    // ===== event loop =====

    loop {
        let Some((key, interest)) = events.next_event() else {
            log::trace!(epoll, "blocking");
            log::flush();
            events.wait(None);
            continue;
        };

        if key & STATIC_ID_MASK == STATIC_ID_MASK {
            match key {
                LISTENER_ID => loop {
                    let conn = match listener.poll_accept() {
                        Ready(Ok(ok)) => ok,
                        Ready(Err(err)) => {
                            log::error!(listener, "{err}");
                            break;
                        }
                        Pending => break,
                    };
                    let (id, client) = clients.insert(conn);
                    events.add(id.to_u64(), client.conn());
                    log::debug!(client, "id={id} connected");
                },
                SIGFD_ID => {
                    let sig = sigfd.read();
                    log::info!(sigfd, "{sig} signal received");
                    break;
                },
                _ => log::error!(epoll, "unknown static key: {key}"),
            }
            continue;
        }

        let id = ClientId::from_u64(key);

        if interest.is_close() {
            let Some(client) = clients.remove(id) else {
                log::warn!(epoll, "unknown dynamic key: {id}");
                continue;
            };
            events.delete(client.conn());
            log::debug!(client, "id={id} disconnected (hup)");
            continue;
        }

        debug_assert!(read_buf.is_empty());
        debug_assert!(write_buf.is_empty());

        let Some(client) = clients.get_mut(id) else {
            log::warn!(epoll, "unknown dynamic key: {id}");
            continue;
        };

        client.buffer_mut().restore(&mut read_buf, &mut write_buf);

        // cope and seeth: https://github.com/rust-lang/rust/issues/31436
        let result = (|| {
            if interest.is_read() {
                let mut client = ClientMut::new(client, &mut write_buf);
                loop {
                    match Frame::from_bytes(&mut read_buf) {
                        Ready(Ok(header)) => handler::router(header, &mut client, &mut compositor)
                            .map_err(|_| FatalError)?,
                        Ready(Err(err)) => {
                            client.send_global_error(err);
                            log::error!(client, "{err}");
                            return Err(FatalError);
                        }
                        Pending => match client.conn().poll_read(&mut read_buf) {
                            Ready(Ok(())) => continue,
                            Ready(Err(err)) => {
                                if !err.is_connection_aborted() {
                                    log::error!(client, "failed to read: {err}");
                                }
                                return Err(FatalError);
                            }
                            Pending => break,
                        },
                    }
                }
            }

            if !write_buf.is_empty() {
                let is_pending = client.conn().poll_write_all(&mut write_buf)?.is_pending();
                match (is_pending, interest.is_write()) {
                    (true, false) => {
                        // first time write pending, add write interest
                        events.modify(id.to_u64(), true, client.conn());
                    }
                    (false, true) => {
                        // previous write pending complete, remove write interest
                        events.modify(id.to_u64(), false, client.conn());
                    }
                    _ => {}
                }
            }

            Ok::<_, FatalError>(())
        })();

        match result {
            Ok(()) => {
                if !read_buf.is_empty() || !write_buf.is_empty() {
                    log::warn!(client, "partial message read: {}, write: {}", read_buf.len(), write_buf.len());

                    // the sad pending bytes cannot stay in shared buffer because it will be used for other
                    // socket, it will be stored in on demand allocation
                    client
                        .buffer_mut()
                        .copy_from(&read_buf, &write_buf);
                }
            },
            Err(_) => {
                if !write_buf.is_empty() {
                    let _ = client
                        .conn()
                        .poll_write_all(&mut write_buf);
                }
                events.delete(client.conn());
                clients.remove(id);
                log::debug!(client, "id={id} disconnected (read)");
            },
        }

        read_buf.clear();
        write_buf.clear();
    }

    Ok(())
}
