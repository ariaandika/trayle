use std::os::fd::RawFd;
use std::ptr::NonNull;

use crate::alloc;

// perhaps this might be simpler to use union but, cheese butter

const PTRALIGN: usize = align_of::<usize>() as usize;
const PTRSIZE: usize = size_of::<usize>() as usize;

const HDRSIZE: usize = size_of::<libc::cmsghdr>() as usize;
const HDRALIGN: usize = align_of::<libc::cmsghdr>() as usize;
const HDRINFD: usize = HDRSIZE / FDSIZE;

const FDSIZE: usize = size_of::<RawFd>() as usize;

const _: () = assert!(PTRSIZE * 2 == HDRSIZE);
const _: () = assert!(PTRALIGN == HDRALIGN);

const fn bytes_to_ptr(value: usize) -> usize {
    value / PTRALIGN
}

// random tbr
const MAX_FD: usize = 64 - HDRINFD;

// cmsg buffer *alignment* need to be the same as `cmsghdr`, in this case is *pointer* alignment,
// but the `CMSG_SPACE` returns size, with the header, is in *bytes* size,
// thus manual layout calculation is required
//
// this represent the allocation size with *pointer* alignment
const TOTAL_SIZE: usize = bytes_to_ptr(unsafe { libc::CMSG_SPACE((MAX_FD * FDSIZE) as u32) as usize });

pub struct FdBuffer {
    ptr: NonNull<RawFd>,
    off: usize,
    len: usize,
}

impl Drop for FdBuffer {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.ptr.sub(self.off + HDRINFD);
            alloc::deallocate::<usize>(ptr.cast());
        }
    }
}

impl FdBuffer {
    /// Note that `FdBuffer` does not close fds on drop.
    pub fn new() -> Self {
        let ptr = alloc::allocate::<usize>(TOTAL_SIZE).cast();
        let ptr = unsafe { ptr.add(HDRINFD) };
        Self {
            ptr,
            off: 0,
            len: 0,
        }
    }

    // pub const fn len(&self) -> usize {
    //     self.len
    // }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn advance_one(&mut self) {
        debug_assert!(self.len != 0);
        self.ptr = unsafe { self.ptr.add(FDSIZE) };
        self.off += 1;
        self.len -= 1;
    }

    /// # Safety
    ///
    /// `cnt` element after the last element must be initialized.
    pub unsafe fn advance_mut(&mut self, cnt: usize) {
        self.len += cnt;
        debug_assert!(self.len <= MAX_FD);
    }

    // /// Returns `true` if there is remaining capacity.
    // pub fn push(&mut self, fd: RawFd) -> bool {
    //     if self.len == MAX_FD {
    //         return false;
    //     }
    //     unsafe {
    //         self.ptr
    //             .as_ptr()
    //             .copy_from_nonoverlapping(fd.to_ne_bytes().as_ptr(), FDSIZE as usize);
    //         self.advance_mut_one();
    //     }
    //     true
    // }

    pub fn pop_front(&mut self) -> Option<RawFd> {
        if self.len == 0 {
            return None;
        }
        let fd = unsafe { self.ptr.cast().read_unaligned() };
        self.advance_one();
        Some(fd)
    }

    /// Clear the buffer, retaining leftover capacity.
    ///
    /// Note that this does not close remaining fds.
    pub fn clear(&mut self) {
        self.ptr = unsafe { self.ptr.sub(self.off * FDSIZE) };
        self.len = 0;
        self.off = 0;
    }
}

impl FdBuffer {
    pub fn as_control(&mut self) -> (*mut std::ffi::c_void, usize) {
        if self.is_empty() {
            return (std::ptr::null_mut(), 0);
        }
        if self.off != 0 {
            unsafe {
                self.ptr = self.ptr.sub(self.off);
                self.ptr
                    .copy_from(self.ptr.add(self.off), self.len);
            }
            self.off = 0;
        }
        let ptr = unsafe { self.ptr.sub(HDRSIZE) };
        (ptr.cast().as_ptr(), self.len * FDSIZE)
    }

    pub fn as_spare_control_mut(&mut self) -> (*mut std::ffi::c_void, usize) {
        let ptr = unsafe { self.ptr.add(self.len) };
        let rem = (MAX_FD - self.off - self.len) * FDSIZE;
        (ptr.cast().as_ptr(), rem)
    }
}
