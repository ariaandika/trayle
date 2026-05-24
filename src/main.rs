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
use std::task::Poll::*;

use buffer::Buffer;
use clients::{ClientId, ClientMut, Clients};
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

fn main() -> ExitCode {
    let _guard = log::init();
    <_>::from(event_loop().is_err() as u8)
}

fn event_loop() -> Result<(), PrintError> {
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

        let Some(mut client) = clients.get_mut(id) else {
            log::warn!(epoll, "unknown dynamic key: {id}");
            continue;
        };

        debug_assert!(read_buffer.is_empty());
        debug_assert!(write_buffer.is_empty());

        client.buffer_mut().copy_to(&mut read_buffer, &mut write_buffer);

        if interest.is_read() {
            let result = loop {
                use wayland::wl_display::encode_error;

                let Some((id, op, len, body)) = wayland::header(&read_buffer) else {
                    match client.conn().poll_read(&mut read_buffer, &mut fds_buffer){
                        Ready(Ok(())) => continue,
                        Ready(Err(err)) => break Err(Some(err)),
                        Pending => break Ok(()),
                    };
                };
                if len < 8 {
                    encode_error(Id::wl_display(), WlError::InvalidSize, &mut write_buffer);
                    log::error!(client, "{}", WlError::InvalidSize);
                    break Err(None);
                }
                let Ok(id) = Id::new(id) else {
                    encode_error(Id::wl_display(), WlError::ZeroId, &mut write_buffer);
                    log::error!(client, "{}", WlError::ZeroId);
                    break Err(None);
                };
                let result = if id.is_display() {
                    handle_wl_display(op, body, &mut client, &mut write_buffer)
                } else {
                    handle_message(id, op, body, &mut client)
                };
                // TODO: checks for recoverable error
                if let Err(err) = result {
                    encode_error(id, err, &mut write_buffer);
                    log::error!(client, "id={id}, {err}");
                    break Err(None);
                }
                read_buffer.advance(len as u32);
            };

            if let Err(err) = result {
                match err {
                    Some(conn::ReadError::ConnectionAborted) => {}
                    Some(read_err) => log::error!(client, "{read_err}"),
                    None => {
                        let _ = client
                            .conn()
                            .poll_write_all(&mut write_buffer, &mut write_fd);
                    },
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

fn handle_wl_display(
    op: u16,
    body: &[u8],
    client: &mut ClientMut<'_>,
    write_buffer: &mut Buffer,
) -> Result<(), WlError> {
    use wayland::wl_display::Op;
    use wayland::WlObject;

    const GLOBALS: [(&str, u16); 9] = [
        ("wl_compositor", 7),
        ("wl_shm", 2),
        ("wl_data_device_manager", 4),
        ("wl_seat", 10),
        ("wl_subcompositor", 1),
        ("wl_fixes", 2),
        ("zwp_linux_dmabuf_v1", 5),
        ("zwp_linux_dmabuf_feedback_v1", 5),
        ("xdg_wm_base", 7),
    ];

    match Op::from_request(op)? {
        Op::Sync(decoder) => {
            decoder.decode(body)?.reply(0, write_buffer);
            log::trace!(client, "<- wl_display::sync");
        }
        Op::GetRegistry(decoder) => {
            let get_registry = decoder.decode(body)?;
            let wl_registry = get_registry.wl_registry();
            client.objects_mut().insert(&wl_registry)?;

            // FEAT: encode globals at startup
            for ((iface, version), i) in GLOBALS.into_iter().zip(0..) {
                wl_registry.global(i, iface, version as u32, &mut *write_buffer);
            }

            log::trace!(client, "<- wl_display::get_registry id={}", wl_registry.id());
        }
    }
    Ok(())
}

fn handle_message(id: Id, op: u16, body: &[u8], client: &mut ClientMut<'_>) -> Result<(), WlError> {
    use wayland::Interface as I;

    let Some(object) = client.objects_mut().get_mut(id) else {
        return Err(WlError::UnknownObject);
    };

    let iface = object.interface();

    match iface {
        I::WlRegistry => {
            use wayland::wl_registry::RequestOp as Op;
            match Op::from_request(op)? {
                Op::Bind(decoder) => {
                    let bind = decoder.decode(body)?;
                    log::trace!(
                        client,
                        "<- wl_registry@bind name={},id={}({},v{})",
                        bind.name,
                        bind.id,
                        bind.id_name,
                        bind.id_version,
                    );
                }
            }
        }
    }

    Ok(())
}

// ===== Error =====

pub struct PrintError;

impl<E: std::fmt::Display> From<E> for PrintError {
    fn from(value: E) -> Self {
        log::error!(setup, "{value}");
        Self
    }
}
