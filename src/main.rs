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
//! # Application
//!
//! - [`wayland`] contains all wayland logic
//!
//! # Util
//!
//! - [`error`] error handling
use std::process::ExitCode;
use std::task::Poll::{self, *};

use buffer::Buffer;
use clients::{Client, ClientId, Clients};
use epoll::Epoll;
use fd_buffer::FdBuffer;
use listener::{Listener, SocketPath};
use sigfd::Sigfd;
use wayland::{Id, WlError};

// ===== os ========
mod errno;
mod epoll;
mod sigfd;
mod conn;
mod listener;
// ===== alloc =====
mod alloc;
mod buffer;
mod fd_buffer;
mod small_buf;
// ===== app =======
mod objects;
mod wayland;
mod clients;
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

fn main() -> ExitCode {
    let _guard = log::init();
    <_>::from(event_loop().is_err() as u8)
}

fn event_loop() -> Result<(), FatalError> {
    // ===== os =====
    let listener = Listener::new(SOCKET_PATH)?;
    let sigfd = Sigfd::new()?;
    let epoll = Epoll::new()?;

    epoll.add(LISTENER_ID, &listener);
    epoll.add(SIGFD_ID, &sigfd);

    // ===== alloc =====
    let mut events_read = 0;
    let mut events = Vec::with_capacity(MAX_EPOLL_EVENT);
    let mut read_buffer = Buffer::with_capacity(1024);
    let mut write_buffer = Buffer::with_capacity(1024);
    let mut read_fd = FdBuffer::new();
    let mut write_fd = FdBuffer::new();

    // ===== app =====
    let mut clients = Clients::with_capacity(8);

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
                            write_fd: &mut write_fd,
                        };
                        match handler::router(header, state, &mut read_fd) {
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
                        match client.conn().poll_read(&mut read_buffer, &mut read_fd){
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
                        .poll_write_all(&mut write_buffer, &mut write_fd);
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
                .poll_write_all(&mut write_buffer, &mut write_fd);
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

pub struct State<'a> {
    client: &'a mut Client,
    write_buffer: &'a mut Buffer,
    #[allow(dead_code)]
    write_fd: &'a mut FdBuffer,
}

struct Message<'a> {
    id: Id,
    op: u16,
    body: &'a [u8],
}

impl<'a> Message<'a> {
    fn from_bytes(bytes: &'a mut Buffer) -> Poll<Result<Self, WlError>> {
        let Some(header) = bytes.first_chunk::<8>() else {
            return Pending;
        };
        // the compiler will remove all the unwraps
        let id = Id::from_ne_bytes(*header[..4].as_array().unwrap())?;
        let op = u16::from_ne_bytes(*header[4..6].as_array().unwrap());
        let len = u16::from_ne_bytes(*header[6..8].as_array().unwrap());
        let Some(msg) = bytes.try_split_to(len as u32) else {
            return Pending;
        };
        let Some(body) = msg.get(8..) else {
            return Ready(Err(WlError::InvalidSize));
        };
        Ready(Ok(Self { id, op, body }))
    }
}
