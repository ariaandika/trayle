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
use std::task::Poll::{self, *};

use buffer::Buffer;
use clients::{Client, ClientId, Clients};
use epoll::Epoll;
use listener::{Listener, SocketPath};
use seat::Seat;
use sigfd::Sigfd;
use wayland::{Id, WlError};

// ===== os ========
mod errno;
mod epoll;
mod sigfd;
mod conn;
mod listener;
mod seat;
// ===== alloc =====
mod alloc;
mod buffer;
mod small_buf;
// ===== collections =====
mod objects;
mod clients;
// ===== protocol =======
mod wayland;
// ===== app =====
mod handler;
// ===== util ====
mod log;

const SOCKET_PATH: SocketPath = SocketPath::new(c"/tmp/wayland-2");

const STATIC_ID_MASK: u64 = i64::MIN as u64;
const LISTENER_ID: u64 = STATIC_ID_MASK | 1;
const SIGFD_ID: u64 = STATIC_ID_MASK | 2;

const MAX_EPOLL_EVENT: usize = 128;

pub struct FatalError;

impl<E: std::fmt::Display> From<E> for FatalError {
    fn from(value: E) -> Self {
        log::error!(setup, "{value}");
        Self
    }
}

pub struct State<'a> {
    client: &'a mut Client,
    write_buffer: &'a mut Buffer,
    seat: &'a Seat,
}

fn main() -> ExitCode {
    let _guard = log::init();
    <_>::from(event_loop().is_err() as u8)
}

fn event_loop() -> Result<(), FatalError> {
    // ===== os =====
    let seat = Seat::new()?;
    let listener = Listener::new(SOCKET_PATH)?;
    let sigfd = Sigfd::new()?;
    let epoll = Epoll::new()?;

    epoll.add(LISTENER_ID, &listener);
    epoll.add(SIGFD_ID, &sigfd);

    // ===== alloc =====
    let mut events_read = 0;
    let mut events = Vec::with_capacity(MAX_EPOLL_EVENT);
    let mut read_buffer = Buffer::new();
    let mut write_buffer = Buffer::new();

    // ===== app =====
    let mut clients = Clients::new();

    // ===== event loop =====

    loop {
        let Some(event) = events.get(events_read) else {
            log::trace!(epoll, "blocking");
            log::flush();
            events_read = 0;
            events.clear();
            let n = epoll.wait(events.spare_capacity_mut(), None);
            // SAFETY: the kernel guarantee that `n` events has been written
            unsafe { events.set_len(n) };
            continue;
        };
        events_read += 1;
        let (key, interest) = event.to_parts();

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
                    let id = clients.insert(conn, &epoll);
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
            match clients.remove(id, &epoll) {
                Some(()) => log::debug!(client, "id={id} disconnected"),
                None => log::error!(epoll, "unknown dynamic key: {id}"),
            }
            continue;
        }

        let Some(client) = clients.get_mut(id) else {
            log::warn!(epoll, "unknown dynamic key: {id}");
            continue;
        };

        debug_assert!(read_buffer.is_empty());
        debug_assert!(write_buffer.is_empty());

        client.buffer_mut().copy_to(&mut read_buffer, &mut write_buffer);

        if interest.is_read() {
            let result = loop {
                use wayland::wl_display::encode_error;
                match Message::from_bytes(&mut read_buffer) {
                    Ready(Ok(header)) => {
                        let id = header.id;
                        let state = State {
                            client,
                            write_buffer: &mut write_buffer,
                            seat: &seat,
                        };
                        match handler::router(header, state) {
                            Ok(()) => {}
                            Err(err) => {
                                encode_error(id, err, &mut write_buffer);
                                log::error!(client, "{err}");
                                break Err(());
                            },
                        }
                    },
                    Ready(Err(err)) => {
                        encode_error(Id::wl_display(), err, &mut write_buffer);
                        log::error!(client, "{err}");
                        break Err(());
                    },
                    Pending => {
                        match client.conn().poll_read(&mut read_buffer){
                            Ready(Ok(())) => continue,
                            Ready(Err(err)) => {
                                log::debug!(client, "{err}");
                                break Err(());
                            },
                            Pending => {
                                break Ok(());
                            },
                        };
                    }
                }
            };

            if result.is_err() {
                if !write_buffer.is_empty() {
                    let _ = client
                        .conn()
                        .poll_write_all(&mut write_buffer);
                }
                read_buffer.clear();
                write_buffer.clear();
                clients.remove(id, &epoll);
                log::debug!(client, "disconnected");
                continue;
            }
        }

        let pending_write = if !write_buffer.is_empty() {
            let result = client
                .conn()
                .poll_write_all(&mut write_buffer);
            match result {
                Ready(Ok(())) => if interest.is_write() {
                    epoll.modify(false, id.to_u64(), client.conn());
                }
                Ready(Err(err)) => {
                    read_buffer.clear();
                    write_buffer.clear();
                    clients.remove(id, &epoll);
                    log::error!(client, "{}", err);
                    log::debug!(client, "disconnecting");
                    continue;
                }
                Pending => if !interest.is_write() {
                    log::warn!(client, "pending message write {}", write_buffer.len());
                    epoll.modify(true, id.to_u64(), client.conn());
                }
            }
            result.is_pending()
        } else {
            false
        };

        let pending_read = !read_buffer.is_empty();

        // the sad pending bytes cannot stay in shared buffer because it will be used for other
        // socket, it will be stored in on demand allocation
        if pending_read | pending_write {
            if pending_read {
                log::warn!(client, "partial message read {}", read_buffer.len());
            }
            client
                .buffer_mut()
                .copy_from(&mut read_buffer, &mut write_buffer);
        }
    }

    Ok(())
}

// ===== wayland =====

struct Message<'a> {
    id: Id,
    op: u16,
    read_buf: &'a mut Buffer,
}

impl<'a> Message<'a> {
    fn from_bytes(read_buf: &'a mut Buffer) -> Poll<Result<Self, WlError>> {
        let Some(header) = read_buf.first_chunk::<8>() else {
            return Pending;
        };
        // the compiler will remove all the unwraps
        let id = Id::from_ne_bytes(*header[..4].as_array().unwrap())?;
        let op = u16::from_ne_bytes(*header[4..6].as_array().unwrap());
        let len = u16::from_ne_bytes(*header[6..8].as_array().unwrap());
        if len < 8 {
            return Ready(Err(WlError::InvalidSize));
        }
        if read_buf.len() < len as usize {
            return Pending;
        }
        read_buf.advance(8);
        Ready(Ok(Self { id, op, read_buf }))
    }
}
