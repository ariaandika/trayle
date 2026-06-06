use std::os::fd::{AsRawFd, FromRawFd};

use crate::sys::errno::simple_errno;

/// An anonymus file.
#[derive(Debug)]
pub struct Memfd(i32);

impl AsRawFd for Memfd {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.0.as_raw_fd()
    }
}

impl Memfd {
    /// Create new `Memfd`.
    pub fn new() -> Result<Self, CreateError> {
        unsafe {
            let fd = libc::memfd_create(c"wayland-keymap".as_ptr(), 0);
            if fd == -1 {
                return Err(CreateError);
            }
            Ok(Self(<_>::from_raw_fd(fd)))
        }
    }

    /// Write bytes to `Memfd`.
    ///
    /// Note that this will block the thread until all bytes written or an error occured.
    pub fn write_all(&self, bytes: &[u8]) -> Result<(), WriteError> {
        let mut written = 0;
        while written < bytes.len() {
            let chunk = &bytes[written..];
            let result =
                unsafe { libc::write(self.as_raw_fd(), chunk.as_ptr().cast(), chunk.len()) };
            let Ok(write) = usize::try_from(result) else {
                return Err(WriteError);
            };
            written += write;
        }
        Ok(())
    }
}

// ===== Error =====

simple_errno! {
    pub CreateError, "failed to create memfd: {}";
    pub WriteError, "failed to write memfd: {}";
}
