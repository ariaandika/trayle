use std::os::fd::{AsRawFd, FromRawFd};

use crate::sys::error::{ErrCode, simple_os_error};

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
    #[inline]
    pub fn new() -> Result<Self, CreateError> {
        unsafe {
            let fd = libc::memfd_create(c"wayland-keymap".as_ptr(), 0);
            if fd == -1 {
                return Err(ErrCode::errno().into());
            }
            Ok(Self(<_>::from_raw_fd(fd)))
        }
    }

    /// Write bytes to `Memfd`.
    ///
    /// Note that this will block the thread until all bytes written or an error occured.
    #[inline]
    pub fn write_all(&self, bytes: &[u8]) -> Result<(), WriteError> {
        let mut written = 0;
        while written < bytes.len() {
            let chunk = &bytes[written..];
            let result =
                unsafe { libc::write(self.as_raw_fd(), chunk.as_ptr().cast(), chunk.len()) };
            let Ok(write) = usize::try_from(result) else {
                return Err(ErrCode::errno().into());
            };
            written += write;
        }
        Ok(())
    }
}

// ===== Error =====

#[derive(Clone, Copy)]
pub struct CreateError(ErrCode);

simple_os_error!(CreateError, "create memfd");

#[derive(Clone, Copy)]
pub struct WriteError(ErrCode);

simple_os_error!(WriteError, "write memfd");
