use std::task::Poll::*;
use todex::sys::buffer;
use todex::sys::listener::{Listener, SocketPath};
use todex::sys::sigfd::Sigfd;
use todex::rt::poller::Poller;

use crate::buffer::BufferPool;
use crate::seat::Seat;
use crate::client::{ClientMut, Clients};
use crate::log;
use crate::error::FatalError;
use crate::compositor::Compositor;

const SOCKET_PATH: SocketPath = SocketPath::new(c"/tmp/wayland-2");
const STATIC_KEY: u64 = i64::MIN as u64;

const LISTENER_KEY: u64 = STATIC_KEY;
const SIGFD_KEY: u64 = STATIC_KEY | 1;

pub fn event_loop() -> Result<(), FatalError> {
    let listener = Listener::new(SOCKET_PATH)?;
    let sigfd = Sigfd::new()?;

    let seat = Seat::new()?;
    let mut compositor = Compositor::new(seat);
    let mut clients = Clients::new();
    let mut buffer = BufferPool::new();

    let mut poll = Poller::new()?;

    poll.add(LISTENER_KEY, &listener);
    poll.add(SIGFD_KEY, &sigfd);

    // ===== event loop =====

    loop {
        let Some((key, interest)) = poll.next_event() else {
            log::debug!(target: "polling", "blocking");
            log::flush();
            poll.wait(None);
            continue;
        };

        if key & STATIC_KEY == STATIC_KEY {
            match key {
                SIGFD_KEY => {
                    log::info!("{} signal received", sigfd.read());
                    break;
                }
                LISTENER_KEY => {
                    while let Ready(result) = listener.poll_accept() {
                        match result {
                            Ok(fd) => {
                                let (id, sock) = clients.insert(fd);
                                poll.add(id, sock);
                                log::debug!(target: format_args!("client#{id}"), "connected");
                            },
                            Err(err) => {
                                log::error!(target: "listener", "{err}")
                            },
                        }
                    }
                },
                _ => {},
            }
            continue;
        }

        debug_assert!(buffer.is_empty());

        let id = key;

        buffer.restore_pending(id as usize);

        let Some(client) = clients.get_mut(id) else {
            log::warn!(target: "polling", "unknown client id from event key: {id}");
            continue;
        };

        client.buffer.restore(&mut buffer.read_buf, &mut buffer.write_buf);

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
            if buffer.store_pending(id as usize) {
                log::warn!(
                    target: format_args!("client#{id}"),
                    "partial message read: {}, write: {}",
                    buffer.read_buf.len(),
                    buffer.write_buf.len()
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

    Ok(())
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
