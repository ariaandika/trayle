use std::task::Poll::*;
use todex::sys::listener::Listener;
use todex::sys::epoll::Epoll;

use crate::client::Clients;
use crate::log;

pub struct ListenerService<'a> {
    listener: &'a Listener,
    epoll: &'a Epoll,
}

impl<'a> ListenerService<'a> {
    pub fn new(listener: &'a Listener, epoll: &'a Epoll) -> Self {
        Self { listener, epoll }
    }

    pub fn serve(&mut self, clients: &mut Clients) {
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
