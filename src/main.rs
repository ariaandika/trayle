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
// ===== app =======
mod objects;
mod wayland;
mod clients;
// ===== util ====
mod log;

const SOCKET_PATH: SocketPath = SocketPath::new(c"/tmp/wayland-2");

const STATIC_ID_MASK: u64 = i64::MIN as u64;
const LISTENER_ID: u64 = STATIC_ID_MASK | 1;
const SIGFD_ID: u64 = STATIC_ID_MASK | 2;

const MAX_EPOLL_EVENT: usize = 128;
const MAX_FD: u32 = 32;
const MAX_FD_SIZE: u32 = MAX_FD * size_of::<i32>() as u32;

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
    let mut fds_buffer = Buffer::with_capacity(MAX_FD_SIZE);
    let mut write_fd = FdBuffer::new::<16>();

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
                match Header::from_bytes(&read_buffer) {
                    Ready(Ok(header)) => {
                        let id = header.id;
                        let total_len = header.body.len() + 8;
                        match header.handle(State {
                            client,
                            write_buffer: &mut write_buffer,
                            read_fd: &mut fds_buffer,
                            write_fd: &mut write_fd,
                        }) {
                            Ok(()) => read_buffer.advance(total_len as u32),
                            Err(err) => {
                                encode_error(id, err, &mut write_buffer);
                                break Err(());
                            },
                        }
                    },
                    Ready(Err(err)) => {
                        encode_error(Id::wl_display(), err, &mut write_buffer);
                        break Err(());
                    },
                    Pending => {
                        match client.conn().poll_read(&mut read_buffer, &mut fds_buffer){
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

struct Header<'a> {
    id: Id,
    op: u16,
    body: &'a [u8],
}

impl<'a> Header<'a> {
    fn from_bytes(bytes: &'a [u8]) -> Poll<Result<Self, WlError>> {
        let Some((header, rest)) = bytes.split_first_chunk::<8>() else {
            return Pending;
        };
        let id = Id::from_ne_bytes(*header[..4].as_array().unwrap())?;
        let op = u16::from_ne_bytes(*header[4..6].as_array().unwrap());
        let len = u16::from_ne_bytes(*header[6..8].as_array().unwrap());
        let Some(body_len) = len.checked_sub(8) else {
            return Ready(Err(WlError::InvalidSize));
        };
        let Some(body) = rest.get(..body_len as usize) else {
            return Pending;
        };
        Ready(Ok(Self { id, op, body }))
    }

    fn handle(self, state: State) -> Result<(), WlError> {
        use wayland::Interface as I;

        if self.id.is_display() {
            return self.handle_wl_display(state);
        }

        let Header { id, op, body } = self;

        let Some(object) = state.client.objects_mut().get_mut(id) else {
            return Err(WlError::UnknownObject);
        };

        let iface = object.interface();

        match iface {
            I::WlRegistry => {
                use wayland::wl_registry::RequestOp as Op;
                match Op::from_request(op)? {
                    Op::Bind(d) => state.handle(d.decode(body)?),
                }
            }
            _ => {
                log::error!(client, "`{iface:?}::{op}` is not yet implemented");
                WlError::todo()
            }
        }
    }

    fn handle_wl_display(self, state: State) -> Result<(), WlError> {
        use wayland::WlObject;
        use wayland::wl_display::Op;

        let Header { op, body, .. } = self;

        match Op::from_request(op)? {
            Op::Sync(decoder) => {
                decoder.decode(body)?.reply(0, state.write_buffer);
                log::trace!(client, "<- wl_display::sync");
            }
            Op::GetRegistry(decoder) => {
                let get_registry = decoder.decode(body)?;
                let wl_registry = get_registry.wl_registry();
                state.client.objects_mut().insert_object(&wl_registry)?;

                // FEAT: encode globals at startup
                for ((iface, version, _), i) in wayland::GLOBALS.into_iter().zip(0..) {
                    wl_registry.global(i, iface, version as u32, state.write_buffer);
                }

                log::trace!(
                    client,
                    "<- wl_display::get_registry id={}",
                    wl_registry.id()
                );
            }
        }
        Ok(())
    }
}

// ===== EventHandler =====

use wayland::wl_registry::Bind;

#[allow(dead_code)]
struct State<'a> {
    client: &'a mut Client,
    write_buffer: &'a mut Buffer,
    read_fd: &'a mut Buffer,
    write_fd: &'a mut FdBuffer,
}

trait EventHandler<Event> {
    fn handle(self, event: Event) -> Result<(), WlError>;
}

impl<'a> EventHandler<Bind<'a>> for State<'a> {
    fn handle(self, bind: Bind<'a>) -> Result<(), WlError> {
        log::trace!(
            client,
            "<- wl_registry@bind {{ name:{}, id:{}, global: ({}, v{}) }}",
            bind.name,
            bind.id,
            bind.id_name,
            bind.id_version,
        );
        let Some((bind_name, version, iface)) = wayland::GLOBALS.get(bind.name as usize) else {
            return Err(WlError::UnknownBind);
        };
        if bind.id_name != *bind_name {
            return Err(WlError::UnknownBind);
        }
        if bind.id_version > *version as u32 {
            return Err(WlError::UnknownBind);
        }
        self.client.objects_mut().insert(bind.id, *iface)?;
        Ok(())
    }
}
