use std::os::fd::OwnedFd;
use todex::collections::slab::Slab;

use crate::client::{ClientState, ClientId};

const INITIAL_CAP: usize = 8;

/// Collections of clients.
pub struct Clients {
    buf: Slab<ClientState>
}

impl Clients {
    pub fn new() -> Self {
        Self {
            buf: Slab::with_capacity(INITIAL_CAP),
        }
    }

    pub fn insert(&mut self, socket: OwnedFd) -> (ClientId, &mut ClientState) {
        let (id, client) = self.buf.insert(ClientState::new(socket));
        (ClientId::from_idx(id), client)
    }

    pub fn get_mut(&mut self, id: ClientId) -> Option<&mut ClientState> {
        self.buf.get_mut(id.idx())
    }

    pub fn remove(&mut self, id: ClientId) -> Option<ClientState> {
        self.buf.remove(id.idx())
    }
}
