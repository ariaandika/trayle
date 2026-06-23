use crate::sys::bytes::Bytes;
use crate::sys::cmsg::Cmsg;
use crate::wayland::primitives::AsObjectId;
use crate::wayland::wire::{AsOpCode, OpCode, Writer};
use crate::wayland::Message;

/// Encode wayland message.
pub trait Encode: Sized + EncodePayload + AsObjectId + AsOpCode {
    /// Encode the message.
    ///
    /// Note that caller must pull the fd from [`EncodePayload::fd`] first before encoding.
    #[inline]
    fn encode(self, writer: Writer) {
        let size = 8 + self.size() as usize;
        let hdr2 = (size as u32) << u16::BITS | Self::OPCODE.to_op() as u32;
        let object_id = self.object_id();
        self.encode_payload(writer.write(object_id).write(hdr2));
    }

    /// Encode the message to [`Bytes`] and [`Cmsg`].
    #[inline]
    fn encode_with(self, write_buf: &mut Bytes, write_fd: &mut Cmsg) {
        if let Some(fd) = self.fd() {
            assert!(write_fd.write_fd(fd));
        }
        let size = 8 + self.size() as usize;
        write_buf.reserve(size);
        self.encode(Writer::new(
            write_buf.spare_capacity_mut().as_mut_ptr().cast::<u8>(),
        ));
        // SAFETY: `Write` implementation guarantee `size` data is initialized
        unsafe { write_buf.advance_mut(size) };
    }
}

/// Encode wayland message payload.
pub trait EncodePayload: Sized {
    /// Returns the size of the payload.
    fn size(&self) -> u16;

    /// Encode message payload.
    ///
    /// Note that `fd` is not written here, caller must get the fd from [`EncodePayload::fd`] first.
    fn encode_payload(self, writer: Writer);

    /// Returns `fd` for this message, if any.
    #[inline]
    fn fd(&self) -> Option<i32> {
        // NOTE: this is assuming that message have maximum of one fd, specification does not say
        // this, but in practice this is true
        None
    }
}

// ===== implementation =====

impl<T: EncodePayload + AsObjectId + AsOpCode> Encode for T {}

impl<T: EncodePayload> EncodePayload for Message<T> {
    #[inline]
    fn size(&self) -> u16 {
        T::size(self.payload())
    }

    #[inline]
    fn encode_payload(self, writer: Writer) {
        T::encode_payload(self.into_payload(), writer)
    }

    #[inline]
    fn fd(&self) -> Option<i32> {
        T::fd(self.payload())
    }
}
