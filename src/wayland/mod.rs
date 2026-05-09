pub use id::Id;
pub use wl_display::WlDisplay;

mod id;
mod wl_display;

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
