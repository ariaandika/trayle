use todex::rt::poller::{Interest, Poller};
use todex::sys::buffer;

use crate::buffer::BufferPool;
use crate::compositor::Compositor;
use crate::{log};
use crate::client::{ClientMut, Clients};

pub struct ClientService {

}

impl ClientService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn serve(
        &mut self,
        id: u64,
        interest: Interest,
        poll: &Poller,
        buffer: &mut BufferPool,
        compositor: &mut Compositor,
        clients: &mut Clients,
    ) {
        // TODO: add flag for pending buffer
        // buffer.restore_pending(id as usize);

        let Some(client) = clients.get_mut(id) else {
            log::warn!(target: "polling", "unknown client id from event key: {id}");
            return;
        };

        client
            .buffer
            .restore(&mut buffer.read_buf, &mut buffer.write_buf);

        // cope and seeth: https://github.com/rust-lang/rust/issues/31436
        let result = (|| {
            if interest.is_close() {
                return Err(HandleError);
            }

            if interest.is_read() {
                let mut client = ClientMut::new(id, client, &mut buffer.write_buf);
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

            if !buffer.write_buf.is_empty() {
                let is_pending = buffer.write_buf.sendmsg(client)?.is_pending();
                match (is_pending, interest.is_write()) {
                    (true, false) => {
                        // first time write pending, add write interest
                        poll.modify(id, true, client);
                    }
                    (false, true) => {
                        // previous write pending complete, remove write interest
                        poll.modify(id, false, client);
                    }
                    _ => {}
                }
            }

            Ok::<_, HandleError>(())
        })();

        if result.is_ok() {
            // the sad pending bytes cannot stay in shared buffer because it will be used for other
            // socket, it will be stored in dedicated storage
            if let Some((read, write)) = buffer.store_pending(id as usize) {
                log::warn!(
                    target: format_args!("client#{id}"),
                    "partial message, read: {read}, write: {write}",
                );
            }
        } else {
            // compositor usually write error before disconnecting
            if !buffer.write_buf.is_empty() {
                let _ = buffer.write_buf.sendmsg(client);
            }
            poll.delete(client);
            clients.remove(id);
            log::debug!(target: format_args!("client#{id}"), "disconnected");
        }

        buffer.clear();
    }
}

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
