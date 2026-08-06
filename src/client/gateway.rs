use std::os::fd::{AsFd, BorrowedFd};
use std::task::Poll::*;
use todex::collections::slab::Slab;
use todex::sys::bytes::Bytes;
use todex::sys::cmsg;
use todex::sys::listener::{Listener, SocketPath};

use crate::buffer::BufferPool;
use crate::client::{ClientId, ClientMut, ClientState};
use crate::error::FatalError;
use crate::poller::{Event, Poller};
use crate::log;

/// Client events reactor.
///
/// This reactor handles:
/// - client connect from [`Listener`]
/// - client I/O from [`ClientMut`]
///
/// This reactor interact with [`Compositor`] to:
/// - dispatch a client message
/// - perform cleanup for client disconnect
pub(crate) struct Gateway {
    clients: Slab<ClientState>,
    listener: Listener,
    pool: BufferPool,
}

impl AsFd for Gateway {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.listener.as_fd()
    }
}

const SOCKET_PATH: SocketPath = SocketPath::new(c"/tmp/wayland-2");
const INIT_CLIENT_CAP: usize = 8;

impl Gateway {
    pub fn new() -> Result<Self, FatalError> {
        Ok(Self {
            clients: Slab::with_capacity(INIT_CLIENT_CAP),
            listener: Listener::new(SOCKET_PATH)?,
            pool: BufferPool::new(),
        })
    }
}

impl Gateway {
    /// Handle a socket I/O event, callback when data is available.
    pub fn dispatch_io<F>(&mut self, event: Event, poll: &Poller, mut callback: F)
    where
        F: FnMut(&mut Bytes, &mut ClientMut),
    {
        let mut id = ClientId::from_raw(event.key);
        let event = event.interest;

        let Some(state) = self.clients.get_mut(id.idx()) else {
            log::warn!(target: "poll", "unknown client id from event key: {id}");
            return;
        };

        if id.is_pending() {
            log::debug!(target: format_args!("client#{id}"), "pending bytes restored");
            self.pool.restore_pending(id.to_raw());
        }

        let mut client = ClientMut {
            id,
            state,
            status: <_>::default(),
            read_fd: &mut self.pool.read_fd,
            write_buf: &mut self.pool.write_buf,
            write_fd: &mut self.pool.write_fd,
        };

        // cope and seeth: https://github.com/rust-lang/rust/issues/31436
        let result = (|| {
            if event.is_close() {
                return Err(HandleError);
            }

            if event.is_read() {
                loop {
                    if !self.pool.read_buf.is_empty() {
                        callback(&mut self.pool.read_buf, &mut client);
                    }
                    if client.status.is_disconnect() {
                        return Err(HandleError);
                    }
                    if client.recvmsg(&mut self.pool.read_buf)?.is_pending() {
                        break;
                    }
                }
            }

            if !client.write_buf.is_empty() {
                let is_pending = client.sendmsg()?.is_pending();
                let was_pending = event.is_write();
                match (is_pending, was_pending) {
                    (true, false) => {
                        id = id.set_pending();
                        // first time write pending, add write interest
                        poll.modify(true, id.to_raw(), &client);
                    }
                    (false, true) => {
                        id = id.unset_pending();
                        // previous write pending complete, remove write interest
                        poll.modify(false, id.to_raw(), &client);
                    }
                    _ => {} // otherwise, double pending or no pending
                }
            }

            Ok(())
        })();

        if result.is_ok() {
            // the sad pending bytes cannot stay in shared buffer because it will be used for other
            // socket, it will be stored in dedicated storage
            if let Some((read, write)) = self.pool.store_pending(id.to_raw()) {
                id = id.set_pending();
                poll.modify(event.is_write(), id.to_raw(), state);
                log::warn!(
                    target: format_args!("client#{id}"),
                    "partial message, read: {read}, write: {write}",
                );
            }
        } else {
            // compositor usually write error before disconnecting
            if !client.write_buf.is_empty() {
                let _ = client.sendmsg();
            }
            poll.delete(&client);
            self.clients.remove(id.idx());
            log::debug!(target: format_args!("client#{id}"), "disconnected");
        }

        self.pool.clear();
    }
}

impl Gateway {
    pub fn dispatch(&mut self, poll: &Poller) {
        while let Ready(result) = self.listener.poll_accept() {
            match result {
                Ok(fd) => {
                    let (id, sock) = self.clients.insert(ClientState::new(fd));
                    ClientId::assert_raw_id(id);
                    poll.add(id as u64, sock);
                    log::debug!(target: format_args!("client#{id}"), "connected");
                }
                Err(err) => {
                    log::error!(target: "listener", "{err}")
                }
            }
        }
    }
}

// ===== Error =====

struct HandleError;

impl From<cmsg::ReadError> for HandleError {
    fn from(err: cmsg::ReadError) -> Self {
        if !err.is_connection_aborted() {
            log::error!("failed to read socket: {err}");
        }
        Self
    }
}

impl From<cmsg::WriteError> for HandleError {
    fn from(err: cmsg::WriteError) -> Self {
        log::error!("failed to write socket: {err}");
        Self
    }
}
