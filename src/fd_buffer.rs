use std::os::fd::RawFd;
use std::ptr::NonNull;
use std::slice;

use crate::alloc;

pub struct FdBuffer {
    ptr: NonNull<u8>,

    // in `RawFd` unit

    off: u32,
    len: u32,
    cap: u32,
}

// perhaps this might be simpler to use union but, cheese butter

// fragment unit is size 8 align 8
// - cmsg header is 2 fragment unit
// - 2 fd is 1 fragment unit

type Fragment = u64;

const fn bytes_to_frag(value: u32) -> u32 {
    value / FRAGALIGN
}

const fn fd_to_frag(value: u32) -> u32 {
    value / FDFRAG
}

const FRAGALIGN: u32 = align_of::<Fragment>() as u32;
const FRAGSIZE: u32 = size_of::<Fragment>() as u32;
const HDRSIZE: u32 = size_of::<libc::cmsghdr>() as u32;
const HDRFRAG: u32 = HDRSIZE / FRAGSIZE;
const FDSIZE: u32 = size_of::<RawFd>() as u32;
const FDFRAG: u32 = FRAGSIZE / FDSIZE;

const _: () = assert!(align_of::<u64>() == align_of::<libc::cmsghdr>());
const _: () = unsafe { assert!(HDRSIZE == libc::CMSG_LEN(0)) };

impl Drop for FdBuffer {
    fn drop(&mut self) {
        unsafe {
            let total_frag = fd_to_frag(self.cap + self.off) + HDRFRAG;
            let ptr = self.ptr
                .cast::<RawFd>()
                .sub(self.off as usize)
                .cast::<Fragment>()
                .sub(HDRFRAG as usize);
            alloc::deallocate(ptr, total_frag);
        }
    }
}

impl FdBuffer {
    /// Note that `FdBuffer` does not close fds on drop.
    pub fn new<const CAP: u32>() -> Self {
        const { assert!(CAP.is_multiple_of(FDFRAG)) }

        // cmsg buffer *alignment* need to be the same as `cmsghdr`, in this case is *pointer size*,
        // but the `CMSG_SPACE` returns size, with the header, in *bytes*,
        // thus manual layout calculation is required
        let total_frag = const {
            let total_bytes = unsafe { libc::CMSG_SPACE(CAP * FDSIZE) };
            debug_assert!(bytes_to_frag(total_bytes) - HDRFRAG == fd_to_frag(CAP));
            bytes_to_frag(total_bytes)
        };
        let ptr = alloc::allocate::<Fragment>(total_frag);
        let ptr = unsafe { ptr.add(HDRFRAG as usize) };
        Self {
            ptr: ptr.cast(),
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

    // fn advance_one(&mut self) {
    //     debug_assert!(self.len != 0);
    //     self.ptr = unsafe { self.ptr.add(FDSIZE as usize) };
    //     self.off += 1;
    //     self.len -= 1;
    //     self.cap -= 1;
    // }
    //
    // fn advance_mut_one(&mut self) {
    //     debug_assert!(self.len != self.cap);
    //     self.len += 1;
    // }
    //
    // /// Returns `true` if there is remaining capacity.
    // pub fn push(&mut self, fd: RawFd) -> bool {
    //     if self.len == self.cap {
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

    pub fn as_cmsg(&self) -> &[u8] {
        debug_assert!(
            self.off == 0,
            "TODO: this should be changed to handle pending write"
        );
        unsafe {
            let base = self.ptr.sub(HDRSIZE as usize);
            slice::from_raw_parts(base.as_ptr(), (self.len * FDSIZE + HDRSIZE) as usize)
        }
    }

    /// Clear the buffer, retaining leftover capacity.
    ///
    /// Note that this does not close remaining fds.
    pub fn clear(&mut self) {
        self.ptr = unsafe { self.ptr.sub((self.off * FDSIZE) as usize) };
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
