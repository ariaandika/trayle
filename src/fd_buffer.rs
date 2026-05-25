use std::os::fd::RawFd;
use std::ptr::NonNull;

use crate::alloc;

// perhaps this might be simpler to use union but, cheese butter

const PTRALIGN: u32 = align_of::<usize>() as u32;
const PTRSIZE: u32 = size_of::<usize>() as u32;

const HDRSIZE: u32 = size_of::<libc::cmsghdr>() as u32;
const HDRALIGN: u32 = align_of::<libc::cmsghdr>() as u32;
const HDRINFD: u32 = HDRSIZE / FDSIZE;

const FDSIZE: u32 = size_of::<RawFd>() as u32;

const _: () = assert!(PTRSIZE * 2 == HDRSIZE);
const _: () = assert!(PTRALIGN == HDRALIGN);

const fn bytes_to_ptr(value: u32) -> u32 {
    value / PTRALIGN
}

// random tbr
const MAX_FD: u32 = 64 - HDRINFD;

// cmsg buffer *alignment* need to be the same as `cmsghdr`, in this case is *pointer* alignment,
// but the `CMSG_SPACE` returns size, with the header, is in *bytes* size,
// thus manual layout calculation is required
//
// this represent the allocation size with *pointer* alignment
const TOTAL_SIZE: u32 = bytes_to_ptr(unsafe { libc::CMSG_SPACE(MAX_FD * FDSIZE) });

pub struct FdBuffer {
    ptr: NonNull<RawFd>,
    off: u32,
    len: u32,
}

impl Drop for FdBuffer {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.ptr.sub((self.off + HDRINFD) as usize);
            alloc::deallocate::<usize>(ptr.cast(), TOTAL_SIZE);
        }
    }
}

impl FdBuffer {
    /// Note that `FdBuffer` does not close fds on drop.
    pub fn new() -> Self {
        let ptr = alloc::allocate::<usize>(TOTAL_SIZE).cast();
        let ptr = unsafe { ptr.add(HDRINFD as usize) };
        Self {
            ptr,
            off: 0,
            len: 0,
        }
    }

    // pub const fn len(&self) -> u32 {
    //     self.len
    // }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    // fn advance_one(&mut self) {
    //     debug_assert!(self.len != 0);
    //     self.ptr = unsafe { self.ptr.add(FDSIZE as usize) };
    //     self.off += 1;
    //     self.len -= 1;
    // }

    /// # Safety
    ///
    /// `cnt` element after the last element must be initialized.
    pub unsafe fn advance_mut(&mut self, cnt: u32) {
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
    //
    // pub fn pop_front(&mut self) -> Option<RawFd> {
    //     if self.len == 0 {
    //         return None;
    //     }
    //     let fd = unsafe { self.ptr.cast().read_unaligned() };
    //     self.advance_one();
    //     Some(fd)
    // }

    /// Clear the buffer, retaining leftover capacity.
    ///
    /// Note that this does not close remaining fds.
    pub fn clear(&mut self) {
        self.ptr = unsafe { self.ptr.sub((self.off * FDSIZE) as usize) };
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
                self.ptr = self.ptr.sub(self.off as usize);
                self.ptr
                    .copy_from(self.ptr.add(self.off as usize), self.len as usize);
            }
            self.off = 0;
        }
        let ptr = unsafe { self.ptr.sub(HDRSIZE as usize) };
        (ptr.cast().as_ptr(), (self.len * FDSIZE) as usize)
    }

    pub fn as_spare_control_mut(&mut self) -> (*mut std::ffi::c_void, usize) {
        let ptr = unsafe { self.ptr.add(self.len as usize) };
        let rem = (MAX_FD - self.off - self.len) * FDSIZE;
        (ptr.cast().as_ptr(), rem as usize)
    }
}
