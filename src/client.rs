use std::os::fd::{AsRawFd, OwnedFd};

use todex::log;
use todex::sys::buffer::{Buffer, SmallBuf};
use todex::collections::slab::Slab;
use todex::compositor::objects::Objects;
use todex::wayland::display;
use todex::wayland::wl_display::{DeleteId, Error};
use todex::wayland::{AsInterface, AsObjectId, AsOpCode, EncodeMessage};
use todex::wayland::{ObjectId, WlError};

// ===== Client =====

/// Client state.
pub struct Client {
    pub socket: OwnedFd,
    pub objects: Objects,
    pub buffer: SmallBuf,
}

impl AsRawFd for Client {
    fn as_raw_fd(&self) -> i32 {
        self.socket.as_raw_fd()
    }
}

// ===== ClientMut =====

pub struct ClientMut<'a> {
    pub id: u64,
    pub state: &'a mut Client,
    pub write_buf: &'a mut Buffer,
}

impl<'a> ClientMut<'a> {
    #[inline]
    pub const fn new(id: u64, state: &'a mut Client, write_buf: &'a mut Buffer) -> Self {
        Self { id, state, write_buf }
    }

    /// Send a message.
    ///
    /// Usually, object has a constructor for its message. The constructor returns the message
    /// wrapped in [`Encodable`] to associate it with object id.
    ///
    /// [`Encodable`]: todex::wayland::Encodable
    #[inline]
    pub fn send<E: EncodeMessage + AsInterface + AsOpCode + display::AsDisplay>(
        &mut self,
        message: E,
    ) {
        log::debug!(
            "client#{} -> {}::{}({})",
            self.id,
            E::INTERFACE_NAME,
            E::OPNAME,
            message.display()
        );
        message.encode_message(self.write_buf);
    }

    /// Send [`DeleteId`] event.
    #[inline]
    pub fn delete_id<O: AsObjectId>(&mut self, object: O) {
        self.send(DeleteId { id: object.object_id() });
    }

    /// Send `wl_display::error` event from [`WlError`].
    #[inline]
    pub fn send_global_error(&mut self, error: WlError) {
        Error::from_wl_error(ObjectId::wl_display(), error).encode_message(self.write_buf);
    }
}

impl AsRawFd for ClientMut<'_> {
    fn as_raw_fd(&self) -> i32 {
        self.state.as_raw_fd()
    }
}

impl<'a> std::ops::Deref for ClientMut<'a> {
    type Target = &'a mut Client;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<'a> std::ops::DerefMut for ClientMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
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
    pub fn insert(&mut self, socket: OwnedFd) -> (u64, &mut Client) {
        let (id, client) = self.buf.insert(Client {
            socket,
            objects: Objects::new(),
            buffer: SmallBuf::default(),
        });
        (id as u64, client)
    }

    #[inline]
    pub fn get_mut(&mut self, id: u64) -> Option<&mut Client> {
        self.buf.get_mut(id as usize)
    }

    #[inline]
    pub fn remove(&mut self, id: u64) -> Option<Client> {
        self.buf.remove(id as usize)
    }
}
