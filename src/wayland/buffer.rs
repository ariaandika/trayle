use std::mem::MaybeUninit;
use std::os::fd::AsRawFd;
use std::ptr::{self, NonNull};
use std::slice;
use std::task::Poll::{self, *};

use crate::sys::errno::{Errno, simple_errno};
use crate::alloc;

const TOTAL_INITIAL_SIZE: usize = 512;
const MAXFD: usize = 16;

const FDSIZE: usize = size_of::<i32>();
const MAXFD_SIZE: usize = FDSIZE * MAXFD;

const INITIAL_CAP: usize = TOTAL_INITIAL_SIZE - MAXFD_SIZE;

// ===== Buffer =====

pub struct MessageBuf {
    ptr: NonNull<u8>,
    off: usize,
    len: usize,
    cap: usize,
    fd_len: usize,
    fd_off: usize,
}

impl Drop for MessageBuf {
    fn drop(&mut self) {
        alloc::deallocate(self.base_ptr());
    }
}

impl MessageBuf {
    #[inline]
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

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    #[inline]
    pub fn advance(&mut self, cnt: usize) {
        assert!(cnt <= self.len);
        // SAFETY: asserted above
        unsafe { self.advance_unchecked(cnt) };
    }

    /// # Safety
    ///
    /// `cnt <= self.len()`
    #[inline]
    pub unsafe fn advance_unchecked(&mut self, cnt: usize) {
        debug_assert!(cnt <= self.len);
        self.ptr = unsafe { self.ptr.add(cnt) };
        self.off += cnt;
        self.len -= cnt;
        self.cap -= cnt;
    }

    /// # Safety
    ///
    /// `cnt` element after the last element must be initialized.
    #[inline]
    pub unsafe fn advance_mut(&mut self, cnt: usize) {
        debug_assert!(self.cap - self.len >= cnt);
        self.len += cnt;
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        if self.cap - self.len < additional {
            self.grow(additional);
        }
    }

    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe {
            slice::from_raw_parts_mut(
                self.ptr.add(self.len).cast().as_ptr(),
                self.cap - self.len
            )
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.ptr = unsafe { self.ptr.sub(self.off) };
        self.cap += self.off;
        self.len = 0;
        self.off = 0;
        self.fd_len = 0;
        self.fd_off = 0;
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

impl MessageBuf {
    #[inline]
    pub fn sendmsg<Fd: AsRawFd>(&mut self, socket: &Fd) -> Poll<Result<(), WriteError>> {
        sendmsg(self, socket.as_raw_fd())
    }

    #[inline]
    pub fn recvmsg<Fd: AsRawFd>(&mut self, socket: &Fd) -> Poll<Result<(), ReadError>> {
        recvmsg(self, socket.as_raw_fd())
    }
}

impl std::ops::Deref for MessageBuf {
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
//
// perhaps this is not true for different architecture, but thats a todo
const _: () = assert!(matches!(
    (size_of::<CmsgBuf>(), align_of::<CmsgBuf>()),
    (CMSG_SPACE, CMSG_ALIGN)
));

fn cmsg_len(fd_len: usize) -> usize {
    unsafe { libc::CMSG_LEN((fd_len * FDSIZE) as u32) as usize }
}

fn cmsg_space(fd_len: usize) -> usize {
    unsafe { libc::CMSG_SPACE((fd_len * FDSIZE) as u32) as usize }
}

const CONTROL_MESSAGE: CmsgBuf = unsafe {
    let mut buf = [std::mem::zeroed::<libc::cmsghdr>(); _];
    buf[0].cmsg_level = libc::SOL_SOCKET;
    buf[0].cmsg_type = libc::SCM_RIGHTS;
    buf
};

// ===== syscall =====

fn sendmsg(buffer: &mut MessageBuf, socket: i32) -> Poll<Result<(), WriteError>> {
    debug_assert!(!buffer.is_empty());
    let mut msg = buffer.as_msghdr();
    loop {
        let Ok(write) = unsafe { libc::sendmsg(socket, msg.as_ptr(), 0) }.try_into() else {
            return match Errno::get() {
                libc::EWOULDBLOCK => Poll::Pending,
                _ => Poll::Ready(Err(WriteError)),
            };
        };
        if msg.advance(write) {
            break;
        }
    }
    Poll::Ready(Ok(()))
}

fn recvmsg(buffer: &mut MessageBuf, socket: i32) -> Poll<Result<(), ReadError>> {
    use ReadError as E;
    debug_assert!(!buffer.spare_capacity_mut().is_empty());

    let mut msghdr = buffer.as_spare_msghdr();

    let Ok(read) = unsafe { libc::recvmsg(socket, msghdr.as_mut_ptr(), 0) }.try_into() else {
        return match Errno::get() {
            libc::EWOULDBLOCK => Poll::Pending,
            _ => Poll::Ready(Err(E::Errno)),
        };
    };
    if read == 0 {
        return Poll::Ready(Err(E::ConnectionAborted));
    }
    if msghdr.is_truncated() {
        return Poll::Ready(Err(E::ControlDataTruncated));
    }
    if unsafe { msghdr.advance_mut(read) } {
        Ready(Ok(()))
    } else {
        Ready(Err(E::ControlDataType))
    }
}

// ===== Error =====

simple_errno! {
    pub WriteError, "failed to write to socket: {}";
}

pub enum ReadError {
    Errno,
    ControlDataType,
    ControlDataTruncated,
    ConnectionAborted,
}

impl ReadError {
    pub fn is_connection_aborted(&self) -> bool {
        matches!(self, Self::ConnectionAborted)
    }
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Errno => write!(f, "failed to read socket: {Errno}"),
            Self::ControlDataType => "unexpected ancillary data type".fmt(f),
            Self::ControlDataTruncated => "ancillary data truncated".fmt(f),
            Self::ConnectionAborted => "connection aborted by the peer".fmt(f),
        }
    }
}

// ===== MsgHdr =====

pub struct MsgHdr<'a> {
    msghdr: libc::msghdr,
    buffer: &'a mut MessageBuf,
}

impl<'a> MsgHdr<'a> {
    fn new(buffer: &'a mut MessageBuf) -> Self {
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
                    unsafe {
                        let dst = libc::CMSG_DATA(&cmsg[0]);
                        dst.copy_from_nonoverlapping(
                            buffer.base_ptr().as_ptr().add(buffer.fd_off * FDSIZE),
                            buffer.fd_len * FDSIZE
                        );
                    };
                    cmsg.as_mut_ptr().cast()
                },
                msg_controllen: if buffer.fd_len == 0 { 0 } else { cmsg_space(buffer.fd_len) },
                msg_flags: 0,
            },
            buffer,
        }
    }

    fn spare(buffer: &'a mut MessageBuf) -> Self {
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
                    cmsg.as_mut_ptr().cast()
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
                let cmsg = &*cmsg_ptr;

                let (libc::SOL_SOCKET, libc::SCM_RIGHTS) = (cmsg.cmsg_level, cmsg.cmsg_type) else {
                    return false;
                };

                let cmsg_data = libc::CMSG_DATA(cmsg);
                let cmsg_len = cmsg.cmsg_len - const { libc::CMSG_LEN(0) as usize };
                let fd_count = cmsg_len / FDSIZE;
                let remaining = MAXFD - self.buffer.fd_off - self.buffer.fd_len;

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

// ===== SmallBuf =====

#[derive(Default)]
pub struct SmallBuf {
    ptr: Option<NonNull<u8>>,
}

impl Drop for SmallBuf {
    fn drop(&mut self) {
        if let Some(ptr) = self.ptr {
            alloc::deallocate(ptr);
        }
    }
}

impl SmallBuf {
    pub fn copy_from(&mut self, read_buf: &MessageBuf, write_buf: &MessageBuf) {
        debug_assert!(self.ptr.is_none());
        debug_assert!(!(read_buf.is_empty() & write_buf.is_empty()));

        const USIZE: usize = size_of::<usize>();
        const HEADER: usize = size_of::<usize>() * 4;
        const FDSPACE: usize = MAXFD_SIZE * 2;

        let read_len = read_buf.len();
        let write_len = write_buf.len();
        let cap = HEADER + FDSPACE + read_len + write_len;
        let ptr = alloc::allocate::<u8>(cap);

        unsafe {
            ptr.cast::<usize>().write_unaligned(read_buf.fd_len);
            ptr.cast::<usize>().add(1).write_unaligned(read_buf.len);
            ptr.add(USIZE * 2)
                .copy_from_nonoverlapping(read_buf.base_ptr(), MAXFD_SIZE + read_buf.len);

            let ptr = ptr.add(USIZE * 2 + MAXFD_SIZE + read_buf.len);

            ptr.cast::<usize>().write_unaligned(write_buf.fd_len);
            ptr.cast::<usize>().add(1).write_unaligned(write_buf.len);
            ptr.add(USIZE * 2)
                .copy_from_nonoverlapping(write_buf.base_ptr(), MAXFD_SIZE + write_buf.len);
        }

        self.ptr = Some(ptr);
    }
    // ```not_rust
    // [
    //     read_fd_len @ usize,
    //     read_len @ usize,
    //     read_buf @ [u8; MAXFD_SIZE + read_len],
    //     write_fd_len @ usize,
    //     write_fd @ [u8; MAXFD_SIZE],
    //     write_buf @ [u8; MAXFD_SIZE + write_len],
    // ]
    // ```
    pub fn restore(&mut self, read_buf: &mut MessageBuf, write_buf: &mut MessageBuf) {
        debug_assert!(read_buf.is_empty() & write_buf.is_empty());
        debug_assert_eq!(
            read_buf.fd_len | read_buf.fd_off | write_buf.fd_len | write_buf.fd_off,
            0
        );

        const USIZE: usize = size_of::<usize>();

        let Some(ptr) = self.ptr else {
            return;
        };

        unsafe {
            let read_fd_len = ptr.cast().read_unaligned();
            let read_len = ptr.cast().add(1).read_unaligned();
            read_buf
                .base_ptr()
                .copy_from_nonoverlapping(ptr.add(USIZE * 2), MAXFD_SIZE + read_len);
            read_buf.fd_len = read_fd_len;
            read_buf.advance_mut(read_len);

            let ptr = ptr.add(USIZE * 2 + MAXFD_SIZE + read_buf.len);

            let write_fd_len = ptr.cast().read_unaligned();
            let write_len = ptr.cast().add(1).read_unaligned();
            write_buf
                .base_ptr()
                .copy_from_nonoverlapping(ptr.add(USIZE * 2), MAXFD_SIZE + write_len);
            write_buf.fd_len = write_fd_len;
            write_buf.advance_mut(write_len);
        }

        self.ptr = None;
    }
}
