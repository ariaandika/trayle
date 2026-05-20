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
use std::task::Poll::*;

use buffer::Buffer;
use clients::{ClientId, ClientMut, Clients};
use epoll::Epoll;
use error::Result;
use fd_buffer::FdBuffer;
use listener::{Listener, SocketPath};
use log::Log;
use sigfd::Sigfd;
use wayland::{Id, WlError};

// ===== os ========
mod errno;
mod epoll;
mod sigfd;
mod conn;
mod listener;
// ===== alloc =====
mod ptr;
mod buffer;
mod fd_buffer;
// ===== app =======
mod objects;
mod wayland;
mod clients;
// ===== util ====
mod log;
mod error;

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
            epoll.info("blocking");
            events_read = 0;
            events.clear();
            let n = epoll.wait(events.spare_capacity_mut(), None)?;
            unsafe { events.set_len(n) };
            epoll.info("complete");
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
                            writeln!(epoll, "{err}");
                            break;
                        }
                        Pending => break,
                    };
                    match clients.add(conn, &epoll) {
                        Ok(client) => client.info("connected"),
                        Err(err) => writeln!(clients, "{err}"),
                    };
                },
                SIGFD_ID => {
                    let sig = sigfd.read();
                    writeln!(sigfd, "{sig} signal received");
                    break;
                },
                _ => writeln!(epoll, "unknown key from epoll: {key}"),
            }
            continue;
        }

        let id = ClientId::from_u64(key);

        if interest.is_close() {
            match clients.remove(id, &epoll) {
                Some(Ok(client)) => writeln!(client, "disconnected"),
                Some(Err(err)) => writeln!(clients, "{err}"),
                None => writeln!(clients, "unknown id from epoll: {id}"),
            }
            continue;
        }

        let Some(mut client) = clients.get_mut(id) else {
            writeln!(clients, "unknown id from epoll: {id}");
            continue;
        };

        if interest.is_write() {
            writeln!(client, "TODO: implement write event")
        }

        loop {
            let Some((id, op, len, body)) = wayland::header(&read_buffer) else {
                match client.conn().poll_read(&mut read_buffer, &mut fds_buffer){
                    Ready(Ok(())) => continue,
                    Ready(Err(err)) => todo!("handle: {err}"),
                    Pending => break,
                };
            };
            let result = 'a: {
                if len < 8 {
                    break 'a Err(WlError::InvalidSize);
                }
                let Ok(id) = Id::new(id) else {
                    break 'a Err(WlError::ZeroId)
                };
                if id.is_display() {
                    handle_wl_display(op, body, &mut write_buffer)
                } else {
                    handle_message(id, op, body, &mut client)
                }
            };
            if let Err(err) = result {
                // TODO: disconnect client
                writeln!(client, "{err}");
                break;
            }
            read_buffer.advance(len as u32);
        };
        if !write_buffer.is_empty() {

            print!("writing: `");
            for &b in write_buffer.as_slice() {
                if b.is_ascii_alphabetic() {
                    print!("{}", b as char);
                } else {
                    print!("\\x{:0>2X}", b);
                }
            }
            println!("`");

            println!("writing: {}", write_buffer.len());
            match client.conn().poll_write_all(&mut write_buffer, &mut write_fd) {
                Ready(Ok(())) => println!("write ok"),
                _ => println!("cannot write"),
            }
            assert!(write_buffer.is_empty());
        }
    }

    Ok(())
}

// ===== wayland =====

fn handle_message(
    id: Id,
    _op: u16,
    _body: &[u8],
    client: &mut ClientMut<'_>,
) -> Result<(), WlError> {
    use wayland::Interface as I;

    let Some(object) = client.objects_mut().get_mut(id) else {
        return Err(WlError::UnknownObject);
    };

    println!("[CLIENT] message: {:?}", object.interface());

    match object.interface() {
        I::WlDisplay => {}
        I::WlRegistry => {}
        I::WlCallback => {}
    }

    Ok(())
}

fn handle_wl_display(op: u16, body: &[u8], write_buffer: &mut Buffer) -> Result<(), WlError> {
    use wayland::wl_display::Op;

    const GLOBALS: [(&str, u16); 1] = [("wl_compositor", 7)];

    match Op::from_request(op)? {
        Op::Sync(decoder) => {
            decoder.decode(body)?.reply(0, write_buffer);
            println!("[CLIENT] message: Sync");
        }
        Op::GetRegistry(decoder) => {
            let get_registry = decoder.decode(body)?;
            let wl_registry = get_registry.wl_registry();
            // FEAT: encode globals at startup
            for ((iface, version), i) in GLOBALS.into_iter().zip(0..) {
                wl_registry.global(i, iface, version as u32, &mut *write_buffer);
            }
            println!("[CLIENT] message: GetRegistry");
        }
    }
    Ok(())
}
