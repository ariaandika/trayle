use std::task::Poll::*;
use todex::rt::poller::Poller;
use todex::sys::listener::Listener;

use crate::client::Clients;
use crate::log;

pub struct ListenerService<'a> {
    listener: &'a Listener,
}

impl<'a> ListenerService<'a> {
    pub fn new(listener: &'a Listener) -> Self {
        Self { listener }
    }

    pub fn serve(&mut self, poll: &Poller, clients: &mut Clients) {
        while let Ready(result) = self.listener.poll_accept() {
            match result {
                Ok(fd) => {
                    let (id, sock) = clients.insert(fd);
                    poll.add(id, sock);
                    log::debug!(target: format_args!("client#{id}"), "connected");
                }
                Err(err) => {
                    log::error!(target: "listener", "{err}")
                }
            }
        }
    }
}
