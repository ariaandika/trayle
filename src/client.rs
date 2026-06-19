use std::os::fd::{AsRawFd, OwnedFd};
use std::task::Poll;
use todex::sys::bytes::Bytes;
use todex::sys::cmsg::{Cmsg, WriteError};
use todex::collections::slab::Slab;
use todex::wayland::display;
use todex::wayland::wl_display::DeleteId;
use todex::wayland::wl_registry::BindData;
use todex::wayland::{AsInterface, AsObjectId, AsOpCode, EncodeMessage};
use todex::compositor::objects::Objects;

use crate::log;

// ===== ClientId =====

const ID_BITS: u64 = u32::MAX as u64;

const MSB: u64 = i64::MIN as u64;
const PENDING_FLAG: u64 = MSB >> 1;

/// Client Id.
#[derive(Debug, Clone, Copy)]
pub struct ClientId(u64);

impl ClientId {
    fn idx(self) -> usize {
        (self.0 & ID_BITS) as usize
    }

    /// Restore client id from raw integer.
    ///
    /// Note that this should only be used to restore id from [`ClientId::to_raw`]. Creating client
    /// id cannot be done externally.
    pub fn from_raw(id: u64) -> Self {
        debug_assert!(id & MSB == 0);
        Self(id)
    }

    /// Convert to raw integer representation.
    ///
    /// The returned integer will always have its most significant bit unset.
    pub fn to_raw(self) -> u64 {
        self.0
    }

    /// Returns `true` if pending flag is set.
    pub fn is_pending(self) -> bool {
        self.0 & PENDING_FLAG == PENDING_FLAG
    }

    // `Interest` also have flag for pending but only on `write` event, thus this bitflag is needed

    /// Set pending flag.
    pub fn set_pending(self) -> Self {
        Self(self.0 | PENDING_FLAG)
    }

    /// Unset pending flag.
    pub fn unset_pending(self) -> Self {
        Self(self.0 & (!PENDING_FLAG))
    }

}

// ===== Client =====

/// Client state.
pub struct ClientState {
    pub socket: OwnedFd,
    pub objects: Objects,
    pub binds: Vec<BindData>,
}

impl AsRawFd for ClientState {
    fn as_raw_fd(&self) -> i32 {
        self.socket.as_raw_fd()
    }
}

// ===== ClientMut =====

/// Client API.
///
/// This state should be created at event time, that is when socket have read or write event.
pub struct ClientMut<'a> {
    pub id: ClientId,
    pub state: &'a mut ClientState,
    pub write_buf: &'a mut Bytes,
    pub write_fd: &'a mut Cmsg,
}

impl<'a> ClientMut<'a> {
    /// Send a message.
    ///
    /// Usually, object has a constructor for its message. The constructor returns the message
    /// wrapped in [`Encodable`] to associate it with object id.
    ///
    /// [`Encodable`]: todex::wayland::Encodable
    pub fn send<E: EncodeMessage + AsInterface + AsOpCode + display::AsDisplay>(
        &mut self,
        message: E,
    ) {
        log::send_message(message, self).encode_message(self.write_buf, self.write_fd);
    }

    /// Send [`DeleteId`] event.
    pub fn delete_id<O: AsObjectId>(&mut self, object: O) {
        self.send(DeleteId {
            id: object.object_id(),
        });
    }

    /// Call [`Cmsg::sendmsg`] on this client.
    pub fn sendmsg(&mut self) -> Poll<Result<(), WriteError>> {
        self.write_fd.sendmsg(self.write_buf, self.state)
    }
}

impl AsRawFd for ClientMut<'_> {
    fn as_raw_fd(&self) -> i32 {
        self.state.as_raw_fd()
    }
}

impl<'a> std::ops::Deref for ClientMut<'a> {
    type Target = &'a mut ClientState;

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
    buf: Slab<ClientState>
}

impl Clients {
    pub fn new() -> Self {
        Self {
            buf: Slab::with_capacity(INITIAL_CAP),
        }
    }

    pub fn insert(&mut self, socket: OwnedFd) -> (ClientId, &mut ClientState) {
        let (id, client) = self.buf.insert(ClientState {
            socket,
            objects: Objects::new(),
            binds: Vec::with_capacity(8),
        });
        assert!(id as u64 & MSB == 0, "client id exhausted");
        (ClientId(id as u64), client)
    }

    pub fn get_mut(&mut self, id: ClientId) -> Option<&mut ClientState> {
        self.buf.get_mut(id.idx())
    }

    pub fn remove(&mut self, id: ClientId) -> Option<ClientState> {
        self.buf.remove(id.idx())
    }
}

// ===== std traits =====

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.idx().fmt(f)
    }
}
