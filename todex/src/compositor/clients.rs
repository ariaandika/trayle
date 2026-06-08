use std::os::fd::{AsRawFd, OwnedFd};

use crate::collections::slab::Slab;
use crate::compositor::objects::{Object, Objects};
use crate::wayland::wl_display::Error;
use crate::wayland::{EncodeMessage, MessageBuf, ObjectId, SmallBuf, WlError, WlObject};

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

impl std::fmt::Debug for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// ===== Client =====

/// Client state.
pub struct Client {
    socket: OwnedFd,
    objects: Objects,
    buffer: SmallBuf,
}

impl Client {
    #[inline]
    pub fn buffer_mut(&mut self) -> &mut SmallBuf {
        &mut self.buffer
    }
}

impl AsRawFd for Client {
    fn as_raw_fd(&self) -> i32 {
        self.socket.as_raw_fd()
    }
}

// ===== ClientMut =====

pub struct ClientMut<'a> {
    id: ClientId,
    state: &'a mut Client,
    write_buf: &'a mut MessageBuf,
}

impl<'a> ClientMut<'a> {
    #[inline]
    pub fn new(id: ClientId, state: &'a mut Client, write_buf: &'a mut MessageBuf) -> Self {
        Self { id, state, write_buf }
    }

    #[inline]
    pub fn objects_mut(&mut self) -> &mut Objects {
        &mut self.state.objects
    }

    #[inline]
    pub fn get_object(&mut self, id: ObjectId) -> Option<&mut Object> {
        self.state.objects.get_mut(id)
    }

    #[inline]
    pub fn insert<O: WlObject>(&mut self, object: &O) -> Result<(), WlError> {
        self.state.objects.insert(object, 0)
    }

    #[inline]
    pub fn insert_with_value<O: WlObject>(&mut self, object: &O, value: usize) -> Result<(), WlError> {
        self.state.objects.insert(object, value)
    }

    #[inline]
    pub fn send<E: EncodeMessage>(&mut self, message: E) {
        message.encode_message(self.write_buf);
    }

    /// Send `wl_display::error` event from [`WlError`].
    #[inline]
    pub fn send_global_error(&mut self, error: WlError) {
        Error::from_wl_error(ObjectId::wl_display(), error).encode_message(self.write_buf);
    }
}

impl<'a> ClientMut<'a> {
    #[inline]
    pub fn log_error(&self, args: std::fmt::Arguments) {
        self.log(crate::log::Level::Error, args);
    }

    #[inline]
    pub fn log_debug(&self, args: std::fmt::Arguments) {
        self.log(crate::log::Level::Debug, args);
    }

    fn log(&self, level: crate::log::Level, args: std::fmt::Arguments) {
        crate::log::logger::log_me(level, format_args!("client#{} ", self.id), args);
    }
}

impl AsRawFd for ClientMut<'_> {
    fn as_raw_fd(&self) -> i32 {
        self.state.as_raw_fd()
    }
}

// ===== Clients =====

const INITIAL_CAP: usize = 8;

/// Collections of clients.
pub struct Clients {
    buf: Slab<Client>
}

impl Clients {
    #[inline]
    pub fn new() -> Self {
        Self {
            buf: Slab::with_capacity(INITIAL_CAP),
        }
    }

    #[inline]
    pub fn insert(&mut self, socket: OwnedFd) -> (ClientId, &mut Client) {
        let (key, client) = self.buf.insert(Client {
            socket,
            objects: Objects::new(),
            buffer: SmallBuf::default(),
        });
        (ClientId(key as u64), client)
    }

    #[inline]
    pub fn get_mut(&mut self, id: ClientId) -> Option<&mut Client> {
        self.buf.get_mut(id.0 as usize)
    }

    #[inline]
    pub fn remove(&mut self, id: ClientId) -> Option<Client> {
        self.buf.remove(id.0 as usize)
    }
}
