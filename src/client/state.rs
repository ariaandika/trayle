use std::ops;
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::task::Poll;

use todex::sys::bytes::Bytes;
use todex::sys::cmsg::{Cmsg, WriteError, ReadError};
use todex::wayland::primitives::AsObjectId;
use todex::wayland::message::WlMessage;
use todex::wayland::interface::wl_display::{DeleteId, Error};
use todex::wayland::wire::Encode;
use todex::wayland::error::WlError;

use crate::client::{ClientId, Objects};
use crate::log;

// ===== Client =====

/// Client state.
pub struct ClientState {
    pub socket: OwnedFd,
    pub objects: Objects,
}

impl ClientState {
    pub(super) fn new(socket: OwnedFd) -> Self {
        Self {
            socket,
            objects: Objects::new(),
        }
    }
}

impl ops::Deref for ClientState {
    type Target = Objects;

    fn deref(&self) -> &Self::Target {
        &self.objects
    }
}

impl ops::DerefMut for ClientState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.objects
    }
}

// ===== ClientStatus =====

#[derive(Clone, Copy, Default)]
pub(crate) enum ClientStatus {
    #[default]
    /// Client ok.
    Ok,
    /// Client wants to disconnect.
    Disconnect,
}

impl ClientStatus {
    pub fn is_disconnect(&self) -> bool {
        matches!(self, Self::Disconnect)
    }
}

// ===== ClientMut =====

/// Client API.
pub struct ClientMut<'a> {
    pub id: ClientId,
    pub state: &'a mut ClientState,
    pub status: ClientStatus,
    pub read_fd: &'a mut Cmsg,
    pub write_buf: &'a mut Bytes,
    pub write_fd: &'a mut Cmsg,
}

impl<'a> ClientMut<'a> {
    /// Send [`DeleteId`] event.
    pub fn delete_id<O: AsObjectId>(&mut self, object: O) {
        self.send(DeleteId {
            id: object.object_id().to_u32(),
        });
    }

    /// Send `wl_display::error` event.
    pub fn send_error<Id: AsObjectId, E: WlError>(&mut self, id: Id, error: E) {
        self.send(Error {
            object_id: id.object_id(),
            code: error.code(),
            message: error.message(),
        });
    }

    /// Send a message.
    ///
    /// Note that automatic version checking is unlikely to be added. Caller must ensure that the
    /// client support following message.
    pub fn send<T: Encode + WlMessage + AsObjectId>(&mut self, msg: T) {
        log::debug!("client#{} -> {}", self.id, msg.display());
        msg.encode_with(self.write_buf, self.write_fd);
    }

    /// Queue for disconnection.
    pub fn disconnect(&mut self) {
        self.status = ClientStatus::Disconnect;
    }

    /// Call write buffer [`Cmsg::sendmsg`].
    pub fn sendmsg(&mut self) -> Poll<Result<(), WriteError>> {
        self.write_fd.sendmsg(self.write_buf, self.state)
    }

    /// Call read buffer [`Cmsg::recvmsg`].
    pub fn recvmsg(&mut self, read_buf: &mut Bytes) -> Poll<Result<(), ReadError>> {
        self.read_fd.recvmsg(read_buf, self.state)
    }
}

// ===== std traits =====

impl AsFd for ClientState {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.socket.as_fd()
    }
}

impl AsFd for ClientMut<'_> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.state.as_fd()
    }
}

impl<'a> ops::Deref for ClientMut<'a> {
    type Target = &'a mut ClientState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<'a> ops::DerefMut for ClientMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}
