use std::os::fd::RawFd;
use std::ptr::{self, NonNull};
use std::slice;

use crate::sys::errno::simple_errno;

/// Memory map.
pub struct Memmap {
    ptr: NonNull<u8>,
    size: usize,
}

impl Drop for Memmap {
    #[inline]
    fn drop(&mut self) {
        // FEAT: drop error logging
        let _res = unsafe { libc::munmap(self.ptr.as_ptr().cast(), self.size) };
    }
}

impl Memmap {
    /// Map memory.
    ///
    /// Note that the fd ownership is not transfered.
    #[inline]
    pub fn new(fd: RawFd, size: usize) -> Result<Self, MapError> {
        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        let ptr = NonNull::new(ptr.cast::<u8>())
            .filter(|e| e.as_ptr().cast() != libc::MAP_FAILED)
            .ok_or(MapError)?;
        Ok(Self { ptr, size })
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.size) }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.size) }
    }
}

impl std::fmt::Debug for Memmap {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Memmap").field(&self.as_slice()).finish()
    }
}

// ===== Error =====

simple_errno! {
    pub MapError, "failed to map memory: {}";
}
