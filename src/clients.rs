use std::ptr::NonNull;

use crate::client::Client;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ClientId(u64);

pub struct Clients {
    _ptr: NonNull<Client>,
    _len: u32,
    _cap: u32,
}

impl Clients {
    pub fn new() -> Self {
        const CAP: u32 = 8;
        let ptr = NonNull::new(Box::into_raw(Box::<[Client]>::new_uninit_slice(
            CAP as usize,
        )))
        .expect("box is non-null");
        Self {
            _ptr: ptr.cast(),
            _len: 0,
            _cap: CAP,
        }
    }

    pub fn insert(&mut self, _client: Client) -> ClientId {
        todo!()
    }

    pub fn get_mut(&mut self, _client_id: ClientId) -> Option<&mut Client> {
        todo!()
    }

    pub fn remove(&mut self, _client_id: ClientId) -> Option<Client> {
        todo!()
    }
}
