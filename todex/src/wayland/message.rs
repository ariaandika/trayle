use crate::sys::buffer::Buffer;
use crate::wayland::{ObjectId, WlError};

// ===== Frame =====

/// Encoded message.
pub struct Frame<'a> {
    /// - guarantee to contains one valid length message
    read_buf: &'a mut Buffer,
}

impl<'a> Frame<'a> {
    #[inline]
    pub fn has_frame(read_buf: &Buffer) -> bool {
        let Some(header) = read_buf.first_chunk::<8>() else {
            return false;
        };
        let len = u16::from_ne_bytes(*header[6..8].as_array().unwrap()) as usize;
        read_buf.len() >= len
    }

    #[inline]
    pub fn new(read_buf: &'a mut Buffer) -> Result<(ObjectId, u16, Self), WlError> {
        let Some(header) = read_buf.first_chunk::<8>() else {
            return Err(WlError::InvalidSize);
        };
        let Some(id) = ObjectId::new(u32::from_ne_bytes(*header.first_chunk().unwrap())) else {
            return Err(WlError::ZeroId);
        };
        let hdr2 = u32::from_ne_bytes(*header.last_chunk().unwrap());
        let len = hdr2 >> u16::BITS;
        if len < 8 {
            return Err(WlError::InvalidSize);
        }
        if read_buf.len() < len as usize {
            return Err(WlError::InvalidSize);
        }
        Ok((id, hdr2 as u16, Self { read_buf }))
    }

    #[inline]
    pub fn pop_fd(&mut self) -> Option<i32> {
        self.read_buf.pop_front_fd()
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
}
