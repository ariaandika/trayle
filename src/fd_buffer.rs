use std::os::fd::RawFd;

use crate::ptr::Ptr;

const FDSIZE: u32 = size_of::<RawFd>() as u32;
const FDALIGN: u32 = align_of::<RawFd>() as u32;

const HDRSIZE: u32 = size_of::<libc::cmsghdr>() as u32;
const HDRALIGN: u32 = align_of::<libc::cmsghdr>() as u32;
const HDRCAP: u32 = HDRSIZE / HDRALIGN;

const _: () = unsafe { assert!(HDRSIZE == libc::CMSG_LEN(0)) };

const ALIGN: u32 = HDRALIGN;
const SIZE_SCALE: u32 = (size_of::<libc::cmsghdr>() / size_of::<RawFd>()) as u32;
const ALIGN_SCALE: u32 = (align_of::<libc::cmsghdr>() / align_of::<RawFd>()) as u32;

pub struct FdBuffer {
    ptr: Ptr<u8>,

    // in `RawFd` unit

    off: u32,
    len: u32,
    cap: u32,
}

// use `u64` to get alignment `HDRALIGN` but size 1
const _: () = assert!(align_of::<u64>() as u32 == HDRALIGN);
type Align = u64;

impl Drop for FdBuffer {
    fn drop(&mut self) {
        let total_cap = (self.cap + self.off) / ALIGN_SCALE + HDRCAP;
        self.ptr
            .sub(self.off * FDSIZE)
            .cast::<Align>()
            .sub(HDRCAP)
            .deallocate(total_cap);
    }
}

impl FdBuffer {
    /// Note that `FdBuffer` does not close fds on drop.
    pub fn new<const CAP: u32>() -> Self {
        const { assert!(CAP.is_multiple_of(ALIGN_SCALE)) }

        // cmsg buffer *alignment* need to be the same as `cmsghdr`, in this case is *pointer size*,
        // but the `CMSG_SPACE` returns size, with the header, in *bytes*,
        // thus manual layout calculation is required
        let total_cap = const {
            let total_bytes = unsafe { libc::CMSG_SPACE(CAP * FDSIZE) };
            debug_assert!((total_bytes - HDRSIZE) / FDSIZE == CAP);
            total_bytes / ALIGN
        };
        let ptr = Ptr::<Align>::allocate(total_cap);

        // then the raw data can be treated as raw bytes
        Self {
            ptr: ptr.add(HDRCAP).cast(), // exclude cmsg header
            off: 0,
            len: 0,
            cap: CAP,
        }
    }

    pub const fn len(&self) -> u32 {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn advance_one(&mut self) {
        self.ptr = self.ptr.add(FDSIZE);
        self.off += 1;
        self.len -= 1;
        self.cap -= 1;
    }

    fn advance_mut_one(&mut self) {
        self.len += 1;
        debug_assert!(self.len <= self.cap);
    }

    /// Returns `true` if there is remaining capacity.
    pub fn push(&mut self, fd: RawFd) -> bool {
        if self.len == self.cap {
            return false;
        }
        self.ptr.copy_from_nonoverlapping(fd.to_ne_bytes().as_ptr(), FDSIZE);
        self.advance_mut_one();
        true
    }

    pub fn pop_front(&mut self) -> Option<RawFd> {
        if self.len == 0 {
            return None;
        }
        let fd = i32::from_ne_bytes(self.ptr.cast::<[u8; 4]>().read());
        self.advance_one();
        Some(fd)
    }

    pub fn as_cmsg(&self) -> &[u8] {
        debug_assert!(self.off == 0);
        let base = self.ptr.sub(HDRSIZE);
        base.as_mut_slice(self.len * FDSIZE + HDRSIZE)
    }

    /// Clear the buffer, retaining leftover capacity.
    ///
    /// Note that this does not close remaining fds.
    pub fn clear(&mut self) {
        self.ptr = self.ptr.sub(self.off * FDSIZE);
        self.cap += self.off;

        self.len = 0;
        self.off = 0;
    }
}

// ===== test =====

#[test]
fn test_fd_buffer() {
    let _fd = FdBuffer::new::<2>();
    let _fd = FdBuffer::new::<4>();
    let _fd = FdBuffer::new::<6>();
    let _fd = FdBuffer::new::<8>();
}
