use todex::rt::poller::{Interest, Poller};
use todex::sys::buffer;

use crate::buffer::BufferPool;
use crate::client::{ClientId, ClientMut, Clients};
use crate::compositor::Compositor;
use crate::log;

pub struct ClientService;

impl ClientService {
    pub fn new() -> Self {
        Self {}
    }
}

impl ClientService {
    pub fn serve(
        &mut self,
        key: u64,
        interest: Interest,
        poll: &Poller,
        buffer: &mut BufferPool,
        clients: &mut Clients,
        compositor: &mut Compositor,
    ) {
        let mut id = ClientId::from_raw(key);

        let Some(client_state) = clients.get_mut(id) else {
            log::warn!(target: "poll", "unknown client id from event key: {id}");
            return;
        };

        if id.is_pending() {
            log::debug!(target: format_args!("client#{id}"), "pending bytes restored");
            buffer.restore_pending(id.to_raw());
        }

        let mut client = ClientMut::new(id, client_state, &mut buffer.write_buf);

        // cope and seeth: https://github.com/rust-lang/rust/issues/31436
        let result = (|| {
            if interest.is_close() {
                return Err(HandleError);
            }

            if interest.is_read() {
                loop {
                    if compositor.has_frame(&buffer.read_buf) {
                        compositor.route(&mut buffer.read_buf, &mut client)?;
                    } else {
                        if buffer.read_buf.recvmsg(&client)?.is_pending() {
                            break;
                        }
                    }
                }
            }

            if !client.write_buf.is_empty() {
                let is_pending = client.sendmsg()?.is_pending();
                let was_pending = interest.is_write();
                match (is_pending, was_pending) {
                    (true, false) => {
                        id = id.set_pending();
                        // first time write pending, add write interest
                        poll.modify(id.to_raw(), true, &client);
                    }
                    (false, true) => {
                        id = id.unset_pending();
                        // previous write pending complete, remove write interest
                        poll.modify(id.to_raw(), false, &client);
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
                poll.modify(id.to_raw(), interest.is_write(), client_state);
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
            clients.remove(id);
            log::debug!(target: format_args!("client#{id}"), "disconnected");
        }

        buffer.clear();
    }
}

// ===== Error =====

struct HandleError;

impl From<buffer::ReadError> for HandleError {
    fn from(err: buffer::ReadError) -> Self {
        if !err.is_connection_aborted() {
            log::error!("failed to read socket: {err}");
        }
        Self
    }
}

impl From<buffer::WriteError> for HandleError {
    fn from(err: buffer::WriteError) -> Self {
        log::error!("failed to write socket: {err}");
        Self
    }
}

impl From<()> for HandleError {
    fn from(_: ()) -> Self {
        Self
    }
}
