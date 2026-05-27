use std::mem::MaybeUninit;
use std::ptr::{self, NonNull};
use std::slice;

use crate::alloc;

const TOTAL_INITIAL_SIZE: usize = 512;
const MAXFD: usize = 16;

const FDSIZE: usize = size_of::<i32>();
const MAXFD_SIZE: usize = FDSIZE * MAXFD;

const INITIAL_CAP: usize = TOTAL_INITIAL_SIZE - MAXFD_SIZE;

// ===== Buffer =====

pub struct Buffer {
    ptr: NonNull<u8>,
    off: usize,
    len: usize,
    cap: usize,
    fd_len: usize,
    fd_off: usize,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        alloc::deallocate(self.base_ptr());
    }
}

impl Buffer {
    pub fn new() -> Self {
        let base_ptr = alloc::allocate(TOTAL_INITIAL_SIZE);
        Self {
            ptr: unsafe { base_ptr.add(MAXFD_SIZE) },
            off: 0,
            len: 0,
            cap: INITIAL_CAP,
            fd_len: 0,
            fd_off: 0,
        }
    }

    fn base_ptr(&self) -> NonNull<u8> {
        unsafe { self.ptr.sub(self.off + MAXFD_SIZE) }
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self, additional: usize) {
        let offset = self.off + MAXFD_SIZE;
        let base_cap = self.cap + offset;
        let base_ptr = unsafe { self.ptr.sub(offset) };
        let new_base_cap = alloc::calc_grow(base_cap, additional);
        let base_ptr = alloc::reallocate(base_ptr, new_base_cap);
        self.ptr = unsafe { base_ptr.add(offset) };
        self.cap = new_base_cap - offset;
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn advance(&mut self, cnt: usize) {
        assert!(cnt <= self.len);
        self.ptr = unsafe { self.ptr.add(cnt) };
        self.off += cnt;
        self.len -= cnt;
        self.cap -= cnt;
    }

    /// # Safety
    ///
    /// `cnt` element after the last element must be initialized.
    pub unsafe fn advance_mut(&mut self, cnt: usize) {
        debug_assert!(self.cap - self.len >= cnt);
        self.len += cnt;
    }

    pub fn try_split_first_chunk<const N: usize>(&mut self) -> Option<&[u8; N]> {
        if N > self.len {
            return None;
        }
        self.advance(N);
        Some(unsafe { self.ptr.sub(N).cast().as_ref() })
    }

    pub fn try_split_to(&mut self, cnt: usize) -> Option<&[u8]> {
        if cnt > self.len {
            return None;
        }
        self.advance(cnt);
        Some(unsafe { slice::from_raw_parts(self.ptr.sub(cnt).as_ptr(), cnt) })
    }

    /// Returns `true` if remaining capacity is sufficient and the data is copied.
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.reserve(slice.len());
        unsafe {
            self.spare_capacity_mut()
                .as_mut_ptr()
                .copy_from_nonoverlapping(slice.as_ptr().cast(), slice.len());
            self.advance_mut(slice.len());
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        if self.cap - self.len < additional {
            self.grow(additional);
        }
    }

    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe {
            slice::from_raw_parts_mut(
                self.ptr.add(self.len).cast().as_ptr(),
                self.cap - self.len
            )
        }
    }

    pub fn clear(&mut self) {
        self.ptr = unsafe { self.ptr.sub(self.off) };
        self.cap += self.off;
        self.len = 0;
        self.off = 0;
    }

    pub fn push_fd(&mut self, fd: i32) -> bool {
        if self.fd_len == MAXFD {
            return false;
        }
        unsafe {
            self.base_ptr()
                .add(self.fd_off + self.fd_len)
                .cast()
                .write_unaligned(fd)
        }
        self.fd_len += 1;
        true
    }

    pub fn pop_front_fd(&mut self) -> Option<i32> {
        if self.fd_len == 0 {
            return None;
        }
        let fd = unsafe { self.base_ptr().add(self.fd_off).cast().read_unaligned() };
        self.fd_off += 1;
        self.fd_len -= 1;
        Some(fd)
    }

    pub fn as_msghdr(&mut self) -> MsgHdr<'_> {
        MsgHdr::new(self)
    }

    pub fn as_spare_msghdr(&mut self) -> MsgHdr<'_> {
        MsgHdr::spare(self)
    }
}

impl std::ops::Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

// ===== CMSG LMAO =====

const IMPOSTOR_CMSGHDR: usize = size_of::<libc::cmsghdr>() / FDSIZE;

// - 1 for the actual header
// - the rest is actually `MAXFD` of fds
type CmsgBuf = [libc::cmsghdr; 1 + IMPOSTOR_CMSGHDR];

// this is the actual size/align when calculating using the macros deez nutz
const CMSG_SPACE: usize = unsafe { libc::CMSG_SPACE(MAXFD_SIZE as u32) as usize };
const CMSG_ALIGN: usize = align_of::<libc::cmsghdr>();

// this prove all statements above
const _: () = assert!(matches!(
    (size_of::<CmsgBuf>(), align_of::<CmsgBuf>()),
    (CMSG_SPACE, CMSG_ALIGN)
));

fn cmsg_len(fd_len: usize) -> usize {
    unsafe { libc::CMSG_LEN((fd_len * FDSIZE) as u32) as usize }
}

const CONTROL_MESSAGE: CmsgBuf = unsafe {
    let mut buf = [std::mem::zeroed::<libc::cmsghdr>(); _];
    buf[0].cmsg_level = libc::SOL_SOCKET;
    buf[0].cmsg_type = libc::SCM_RIGHTS;
    buf
};

// ===== MsgHdr =====

pub struct MsgHdr<'a> {
    msghdr: libc::msghdr,
    buffer: &'a mut Buffer,
}

impl<'a> MsgHdr<'a> {
    fn new(buffer: &'a mut Buffer) -> Self {
        Self {
            msghdr: libc::msghdr {
                msg_name: ptr::null_mut(),
                msg_namelen: 0,
                msg_iov: &mut libc::iovec {
                    iov_base: buffer.ptr.as_ptr().cast(),
                    iov_len: buffer.len,
                },
                msg_iovlen: 1,
                msg_control: if buffer.fd_len == 0 {
                    ptr::null_mut()
                } else {
                    let mut cmsg = CONTROL_MESSAGE;
                    cmsg[0].cmsg_len = cmsg_len(buffer.fd_len);
                    &raw mut cmsg as *mut _
                },
                msg_controllen: if buffer.fd_len == 0 { 0 } else { CMSG_SPACE },
                msg_flags: 0,
            },
            buffer,
        }
    }

    fn spare(buffer: &'a mut Buffer) -> Self {
        let spare = buffer.spare_capacity_mut();
        Self {
            msghdr: libc::msghdr {
                msg_name: ptr::null_mut(),
                msg_namelen: 0,
                msg_iov: &mut libc::iovec {
                    iov_base: spare.as_mut_ptr().cast(),
                    iov_len: spare.len(),
                },
                msg_iovlen: 1,
                msg_control: {
                    let mut cmsg = CONTROL_MESSAGE;
                    &raw mut cmsg as *mut _
                },
                msg_controllen: CMSG_SPACE,
                msg_flags: 0,
            },
            buffer,
        }
    }

    pub const fn as_ptr(&self) -> *const libc::msghdr {
        &raw const self.msghdr
    }

    pub const fn as_mut_ptr(&mut self) -> *mut libc::msghdr {
        &raw mut self.msghdr
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn is_truncated(&self) -> bool {
        self.msghdr.msg_flags & libc::MSG_CTRUNC == libc::MSG_CTRUNC
    }
}

impl<'a> MsgHdr<'a> {
    /// Advance the buffer after `sendmsg` call.
    ///
    /// Returns `true` if buffer has fully written.
    ///
    /// `write` must be the successfull value returned from `sendmsg` call.
    pub fn advance(&mut self, write: usize) -> bool {
        self.buffer.advance(write);

        if self.is_empty() {
            // reset the indexes
            self.buffer.clear();
            self.buffer.fd_len = 0;
            self.buffer.fd_off = 0;
            return true;
        }

        // update the advance message buffer
        let iov_mut = unsafe { &mut *self.msghdr.msg_iov };
        iov_mut.iov_base = self.buffer.as_ptr() as *mut _;
        iov_mut.iov_len = self.buffer.len();

        // Ancillary data is received as if it were queued along with the first normal data octet in
        // the segment (if any).
        //
        // https://unix.stackexchange.com/questions/185011/what-happens-with-unix-stream-ancillary-data-on-partial-reads
        //
        // unset the ancillary data
        self.msghdr.msg_control = ptr::null_mut();
        self.msghdr.msg_controllen = 0;

        false
    }

    /// Mark `read` size data as initialized.
    ///
    /// This will also copy the ancillary data. Returns `true` if this is successfull.
    ///
    /// # Safety
    ///
    /// `read` size after the last element must be initialized.
    pub unsafe fn advance_mut(&mut self, read: usize) -> bool {
        unsafe {
            let mut cmsg_ptr = libc::CMSG_FIRSTHDR(&self.msghdr);
            while !cmsg_ptr.is_null() {
                let cmsg = cmsg_ptr.read_unaligned();

                let (libc::SOL_SOCKET, libc::SCM_RIGHTS) = (cmsg.cmsg_level, cmsg.cmsg_type) else {
                    return false;
                };

                let cmsg_data = libc::CMSG_DATA(&cmsg);
                let cmsg_len = cmsg.cmsg_len - const { libc::CMSG_LEN(0) as usize };
                let fd_count = cmsg_len / FDSIZE;
                let remaining = MAXFD - self.buffer.fd_len;

                if fd_count > remaining {
                    return false;
                }

                self.buffer
                    .base_ptr()
                    .as_ptr()
                    .add(self.buffer.fd_off + self.buffer.fd_len)
                    .cast::<u8>()
                    .copy_from_nonoverlapping(cmsg_data, cmsg_len);
                self.buffer.fd_len += fd_count;

                debug_assert!(self.buffer.fd_len <= MAXFD);

                cmsg_ptr = libc::CMSG_NXTHDR(&self.msghdr, cmsg_ptr);
            }

            self.buffer.advance_mut(read);
        }

        true
    }
}
