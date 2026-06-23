use crate::sys::bytes::Bytes;
use crate::sys::cmsg::Cmsg;
use crate::wayland::primitives::ObjectId;
use crate::wayland::wire::{Decode, DecodeError};

use DecodeError as E;

/// Raw bytes that contains a message.
pub struct Frame<'a> {
    /// - guarantee to contains one valid length message
    read_buf: &'a mut Bytes,
    read_fd: &'a mut Cmsg,
}

impl<'a> Frame<'a> {
    #[inline]
    pub fn has_frame(read_buf: &Bytes) -> bool {
        let Some(header) = read_buf.first_chunk::<8>() else {
            return false;
        };
        let len = u16::from_ne_bytes(*header[6..8].as_array().unwrap()) as usize;
        read_buf.len() >= len
    }

    #[inline]
    pub fn new(read_buf: &'a mut Bytes, read_fd: &'a mut Cmsg) -> Result<(ObjectId, u16, Self), DecodeError> {
        let Some(header) = read_buf.first_chunk::<8>() else {
            return Err(E::InsufficientSize);
        };
        let Some(id) = ObjectId::new(u32::from_ne_bytes(*header.first_chunk().unwrap())) else {
            return Err(E::ZeroId);
        };
        let hdr2 = u32::from_ne_bytes(*header.last_chunk().unwrap());
        let len = hdr2 >> u16::BITS;
        if len < 8 {
            return Err(E::InsufficientSize);
        }
        if read_buf.len() < len as usize {
            return Err(E::InsufficientSize);
        }
        Ok((id, hdr2 as u16, Self { read_buf, read_fd }))
    }

    #[inline]
    pub fn pop_fd(&mut self) -> Option<i32> {
        self.read_fd.read_fd()
    }

    #[inline]
    pub fn body(self) -> &'a [u8] {
        let ptr = self.read_buf.as_ptr();
        unsafe {
            // SAFETY: invariant
            let len = ptr.add(6).cast::<u16>().read_unaligned() as usize;
            // SAFETY: invariant
            self.read_buf.advance_unchecked(len);
            std::slice::from_raw_parts(ptr.add(8), len - 8)
        }
    }

    /// Decode message.
    #[inline]
    pub fn decode<D: Decode>(self) -> Result<D::Output<'a>, DecodeError> {
        D::decode_frame(self)
    }
}

// ===== FrameError =====

#[derive(Debug, Clone, Copy)]
pub enum FrameError {
    /// Insufficient message size.
    InsufficientSize,
    /// Excessive message size.
    ExcessiveSize,
    /// Invalid object id of `0`.
    ZeroId,
}

impl FrameError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::InsufficientSize => "insufficient message size",
            Self::ExcessiveSize => "excessize message size",
            Self::ZeroId => "invalid object id of `0`",
        }
    }
}
