pub use id::Id;
pub use wl_display::WlDisplay;

mod id;
mod wl_display;

/// `(id, op, len)`
#[allow(clippy::type_complexity)]
pub fn split_header(bytes: &[u8]) -> Option<((u32, u16, u16), &[u8])> {
    let (header, rest) = bytes.split_first_chunk::<8>()?;
    unsafe {
        let ptr = header.as_ptr();
        let id = u32::from_ne_bytes(*ptr.cast::<[u8; _]>());
        let op = u16::from_ne_bytes(*ptr.add(4).cast::<[u8; _]>());
        let len = u16::from_ne_bytes(*ptr.add(6).cast::<[u8; _]>());
        Some(((id, op, len), rest))
    }
}

#[repr(u16)]
pub enum Interface {
    WlDisplay,
}

/// Writable type.
///
/// # Safety
///
/// Implementor must ensure that the pointer returned from [`spare`] is valid for write until given
/// length.
///
/// [`spare`]: Self::spare
pub unsafe trait Write {
    /// Returns writable memory.
    ///
    /// # Safety
    ///
    /// Caller must initialize the returned pointer until `len` bytes.
    unsafe fn spare(&mut self, len: usize) -> *mut u8;
}

// ===== blanket implementation =====

unsafe impl<W: Write> Write for &mut W {
    unsafe fn spare(&mut self, len: usize) -> *mut u8 {
        unsafe { W::spare(self, len) }
    }
}
