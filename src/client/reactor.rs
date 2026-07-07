use std::task::Poll::Ready;
use todex::sys::epoll::Epoll;
use todex::sys::cmsg;
use todex::poller::Event;
use todex::sys::listener::Listener;

use crate::buffer::BufferPool;
use crate::client::{ClientId, ClientMut, Clients};
use crate::compositor::Compositor;
use crate::log;

pub struct ClientReactor<'a> {
    epoll: &'a Epoll,
    listener: &'a Listener,
}

impl<'a> ClientReactor<'a> {
    pub fn new(epoll: &'a Epoll, listener: &'a Listener) -> Self {
        Self { epoll, listener }
    }
}

impl ClientReactor<'_> {
    pub fn handle_socket(
        &mut self,
        event: Event,
        buffer: &mut BufferPool,
        clients: &mut Clients,
        compositor: &mut Compositor,
    ) {
        let mut id = ClientId::from_raw(event.key);

        let Some(state) = clients.get_mut(id) else {
            log::warn!(target: "poll", "unknown client id from event key: {id}");
            return;
        };

        if id.is_pending() {
            log::debug!(target: format_args!("client#{id}"), "pending bytes restored");
            buffer.restore_pending(id.to_raw());
        }

        let mut client = ClientMut {
            id,
            state,
            read_fd: &mut buffer.read_fd,
            write_buf: &mut buffer.write_buf,
            write_fd: &mut buffer.write_fd,
        };

        // cope and seeth: https://github.com/rust-lang/rust/issues/31436
        let result = (|| {
            if event.interest.is_close() {
                return Err(HandleError);
            }

            if event.interest.is_read() {
                loop {
                    if !buffer.read_buf.is_empty()
                        && compositor
                            .message(&mut buffer.read_buf, &mut client)
                            .is_disconnect()
                    {
                        return Err(HandleError);
                    }
                    if client.recvmsg(&mut buffer.read_buf)?.is_pending() {
                        break;
                    }
                }
            }

            if !client.write_buf.is_empty() {
                let is_pending = client.sendmsg()?.is_pending();
                let was_pending = event.interest.is_write();
                match (is_pending, was_pending) {
                    (true, false) => {
                        id = id.set_pending();
                        // first time write pending, add write interest
                        self.epoll.modify(true, id.to_raw(), &client);
                    }
                    (false, true) => {
                        id = id.unset_pending();
                        // previous write pending complete, remove write interest
                        self.epoll.modify(false, id.to_raw(), &client);
                    }
                    _ => {} // otherwise, double pending or no pending
                }
            }

            Ok(())
        })();

        if result.is_ok() {
            // the sad pending bytes cannot stay in shared buffer because it will be used for other
            // socket, it will be stored in dedicated storage
            if let Some((read, write)) = buffer.store_pending(id.to_raw()) {
                id = id.set_pending();
                self.epoll
                    .modify(event.interest.is_write(), id.to_raw(), state);
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
            self.epoll.delete(&client);
            clients.remove(id);
            log::debug!(target: format_args!("client#{id}"), "disconnected");
        }

        buffer.clear();
    }
}

impl ClientReactor<'_> {
    pub fn handle_listener(&mut self, clients: &mut Clients) {
        while let Ready(result) = self.listener.poll_accept() {
            match result {
                Ok(fd) => {
                    let (id, sock) = clients.insert(fd);
                    self.epoll.add(id.to_raw(), sock);
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

impl From<()> for HandleError {
    fn from(_: ()) -> Self {
        Self
    }
}

