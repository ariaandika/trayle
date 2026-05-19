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
use fd_buffer::FdBuffer;
use listener::{Listener, SocketPath};
use sigfd::Sigfd;
use wayland::{Id, WlError};

// ===== shared ====
mod error;
// ===== os ========
mod errno;
mod conn;
mod listener;
mod epoll;
mod sigfd;
// ===== alloc =====
mod ptr;
mod buffer;
mod fd_buffer;
// ===== app =======
mod wayland;
mod client;
mod clients;

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
    let mut write_buffer = Buffer::with_capacity(1024);
    let mut fds_buffer = Buffer::with_capacity(MAX_FD_SIZE);
    let mut write_fd = FdBuffer::new::<16>();

    // ===== app =====
    let mut clients = Clients::with_capacity(8);

    // ===== event loop =====

    // separate closure to act as a `try` block while capturing all required state
    //
    // cope and seethe: https://github.com/rust-lang/rust/issues/31436
    let mut handle = |id: u64, interest: epoll::Interest| {
        if id & STATIC_ID_MASK == STATIC_ID_MASK {
            return match id {
                LISTENER_ID => loop {
                    let conn = ready!(listener.poll_accept()).cx::<Listener>("accept client")?;
                    let new_id = clients.peek_id();
                    epoll.add_read(new_id, &conn).cx::<Epoll>("add client")?;
                    clients.insert(Client::new(new_id, conn));
                    println!("[CLIENT] connected {:?}", Clients::destruct_id(new_id));
                },
                SIGFD_ID => Ok(Some(sigfd.read())),
                _ => epoll.err(UnknownKey, "handle event"),
            };
        }

        if interest.is_close() {
            let Some(client) = clients.remove(id) else {
                return clients.err(UnknownId, "remove client");
            };
            epoll.remove(&client).cx::<Epoll>("remove client")?;
            println!("[CLIENT] close");
            return Ok(None);
        }

        let Some(client) = clients.get_mut(id) else {
            return clients.err(UnknownId, "get client");
        };
        loop {
            ready!(client.conn().poll_read(&mut read_buffer, &mut fds_buffer)).cx::<Client>("read socket")?;

            while let Some((id, op, len)) = wayland::header(&read_buffer) {
                if len < 8 {
                    todo!("error: len less than 8")
                }
                let Some(body) = read_buffer.get(8..len as usize) else {
                    break;
                };
                let Some(id) = Id::new(id) else {
                    todo!("invalid id")
                };
                let result = if id.is_display() {
                    handle_wl_display(op, body, &mut write_buffer)
                } else {
                    handle_message(id, op, body, client)
                };
                if let Err(err) = result {
                    println!("[WL_ERROR]: {err}");
                }
                if !write_buffer.is_empty() {
                    dbg!(&*write_buffer);
                    match client.conn().poll_write_all(&mut write_buffer, &mut write_fd) {
                        Ready(Ok(())) => println!("write ok"),
                        _ => println!("cannot write"),
                    }
                    dbg!(&*write_buffer);
                }
                read_buffer.advance(len as u32);
            }
        }
    };

    loop {
        let Some(event) = events.get(events_i) else {
            println!("[EPOLL] blocking");
            events_i = 0;
            events.clear();
            let n = epoll.wait(events.spare_capacity_mut(), None)?;
            unsafe { events.set_len(n) };
            println!("[EPOLL] complete");
            continue;
        };
        events_i += 1;
        let (id, interest) = event.to_parts();
        match handle(id, interest) {
            Ok(sig) => if let Some(sig) = sig {
                println!("[{}] received {sig} signal", Sigfd::NAME);
                break;
            }
            Err(err) => eprintln!("{err}"),
        }
    }

    Ok(())
}

// ===== wayland =====

fn handle_message(
    id: Id,
    op: u16,
    body: &[u8],
    client: &mut Client,
) -> Result<(), WlError> {
    todo!()
}

fn handle_wl_display(
    op: u16,
    body: &[u8],
    write_buffer: &mut Buffer,
) -> Result<(), WlError> {
    use wayland::wl_display::Op;
    match Op::from_request(op)? {
        Op::Sync(decoder) => {
            decoder.decode(body)?.encode_callback(69, write_buffer);
            println!("[DEBUG] Sync");
        }
        Op::GetRegistry(decoder) => {
            let get_registry = decoder.decode(body)?;
            println!("[DEBUG] {get_registry:?}");
        }
    }
    Ok(())
}

macro_rules! ready {
    ($e:expr) => {
        match $e {
            Ready(ok) => ok,
            Pending => break Ok(None),
        }
    };
}

use ready;

// ===== Error Util =====

struct UnknownId;
struct UnknownKey;

macro_rules! impl_subject {
    ($t:ty, $n:literal) => {
        impl Subject for $t {
            const NAME: &'static str = $n;
        }
    };
}
impl_subject!(Epoll, "EPOLL");
impl_subject!(Sigfd, "SIGFD");
impl_subject!(Listener, "LISTENER");
impl_subject!(Client, "CLIENT");
impl_subject!(Clients, "CLIENTS");

trait Subject {
    const NAME: &'static str;

    fn err<T, R: Into<Repr>>(&self, err: R, m: &'static str) -> Result<T, HandleError> {
        Err(HandleError::new(Self::NAME, m, err.into()))
    }
}

trait HandleErrorExt<T> {
    fn cx<S: Subject>(self, m: &'static str) -> Result<T, HandleError>;
}

impl<S: Subject> Subject for &S {
    const NAME: &'static str = S::NAME;
}

impl<T, E: Into<Repr>> HandleErrorExt<T> for Result<T, E> {
    fn cx<S: Subject>(self, m: &'static str) -> Result<T, HandleError> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(HandleError::new(S::NAME, m, err.into()))
        }
    }
}

impl<T> HandleErrorExt<T> for Option<T> {
    fn cx<S: Subject>(self, m: &'static str) -> Result<T, HandleError> {
        match self {
            Some(ok) => Ok(ok),
            None => Err(HandleError::new(S::NAME, m, Repr::None)),
        }
    }
}

// ===== Handle Error =====

struct HandleError {
    subject: &'static str,
    message: &'static str,
    repr: Repr,
}

enum Repr {
    Errno,
    UnknownId,
    UnknownKey,
    MsgError(conn::MsgError),
    None,
}

macro_rules! impl_into_repr {
    ($t:ty, $r:ident) => {
        impl From<$t> for Repr {
            fn from(_: $t) -> Self { Self::$r }
        }
    };
}
impl_into_repr!(errno::Errno, Errno);
impl_into_repr!(UnknownId, UnknownId);
impl_into_repr!(UnknownKey, UnknownKey);

impl From<conn::MsgError> for Repr {
    fn from(value: conn::MsgError) -> Self {
        Self::MsgError(value)
    }
}

impl HandleError {
    fn new(subject: &'static str, message: &'static str, repr: Repr) -> Self {
        Self { subject, message, repr }
    }
}

impl std::fmt::Display for HandleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] failed to {}", self.subject, self.message)?;
        match &self.repr {
            Repr::Errno => write!(f, ": {}", std::io::Error::last_os_error()),
            Repr::UnknownId => write!(f, ": unrecognized ID"),
            Repr::UnknownKey => write!(f, ": unrecognized key"),
            Repr::MsgError(err) => err.fmt(f),
            Repr::None => Ok(()),
        }
    }
}
