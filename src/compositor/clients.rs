use crate::sys::conn::Connection;
use crate::collections::slab::Slab;
use crate::collections::buffer::{Buffer, SmallBuf};
use crate::compositor::objects::Objects;
use crate::wayland::wl_display;
use crate::wayland::{Encode, Id, Object, WlError};

// ===== ClientId =====

/// Unique id for client.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ClientId(u64);

impl ClientId {
    /// Note that this should only be used to restore id from raw integer.
    ///
    /// To create new id, use `Clients` methods.
    pub fn from_u64(int: u64) -> Self {
        Self(int)
    }

    pub fn to_u64(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.0 as u32).fmt(f)
    }
}

// ===== Client =====

// Client state.
pub struct Client {
    conn: Connection,
    objects: Objects,
    buffer: SmallBuf,
}

impl Client {
    #[inline]
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    #[inline]
    pub fn objects_mut(&mut self) -> &mut Objects {
        &mut self.objects
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut SmallBuf {
        &mut self.buffer
    }

    #[inline]
    pub fn send_global_error(&mut self, error: WlError, write_buf: &mut Buffer) {
        wl_display::error_from(Id::wl_display(), error).encode_to(write_buf);
    }
}

// ===== ClientMut =====

pub struct ClientMut<'a> {
    state: &'a mut Client,
    write_buf: &'a mut Buffer,
}

impl<'a> ClientMut<'a> {
    #[inline]
    pub fn new(state: &'a mut Client, write_buf: &'a mut Buffer) -> Self {
        Self { state, write_buf }
    }

    #[inline]
    pub fn conn(&self) -> &Connection {
        &self.state.conn
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut SmallBuf {
        &mut self.state.buffer
    }

    #[inline]
    pub fn objects_mut(&mut self) -> &mut Objects {
        &mut self.state.objects
    }

    #[inline]
    pub fn insert_object<O: Object>(&mut self, object: &O) -> Result<(), WlError> {
        self.state.objects.insert_object(object)
    }

    #[inline]
    pub fn send<E: Encode>(&mut self, message: E) {
        message.encode_to(self.write_buf);
    }

    #[inline]
    pub fn send_global_error(&mut self, error: WlError) {
        wl_display::error_from(Id::wl_display(), error).encode_to(self.write_buf);
    }
}

// ===== Clients =====

const INITIAL_CAP: usize = 8;

/// Collections of clients.
pub struct Clients {
    buf: Slab<Client>
}

impl Clients {
    pub fn new() -> Self {
        Self {
            buf: Slab::with_capacity(INITIAL_CAP),
        }
    }

    pub fn insert(&mut self, conn: Connection) -> (ClientId, &mut Client) {
        let (key, client) = self.buf.insert(Client {
            conn,
            objects: Objects::new(),
            buffer: SmallBuf::default(),
        });
        (ClientId(key as u64), client)
    }

    pub fn get_mut(&mut self, id: ClientId) -> Option<&mut Client> {
        self.buf.get_mut(id.0 as usize)
    }

    pub fn remove(&mut self, id: ClientId) -> Option<Client> {
        self.buf.remove(id.0 as usize)
    }
}
