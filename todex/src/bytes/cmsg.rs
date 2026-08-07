use std::mem::MaybeUninit;
use std::os::fd::{AsFd, AsRawFd};
use std::ptr;
use std::task::Poll::{self, *};

use crate::bytes::Bytes;
use crate::sys::error::{ErrCode, simple_os_error};

const FDSIZE: usize = size_of::<i32>();
const CMSGHDRSIZE: usize = size_of::<libc::cmsghdr>();

const MAXFD: usize = 16;
const MAXCAP: usize = MAXFD * FDSIZE;

/// Buffer for control message containing file descriptors.
pub struct Cmsg {
    buf: Box<[MaybeUninit<i32>; MAXFD]>,
    len: usize,
    off: usize,
}

impl Cmsg {
    /// Create new `Cmsg`.
    #[inline]
    pub fn new() -> Self {
        Self {
            buf: Box::new([MaybeUninit::uninit(); MAXFD]),
            len: 0,
            off: 0,
        }
    }

    fn buf(&self) -> &[MaybeUninit<i32>] {
        // SAFETY: `off <= self.buf.len()`
        unsafe { self.buf.get_unchecked(self.off..) }
    }

    fn buf_mut(&mut self) -> &mut [MaybeUninit<i32>] {
        // SAFETY: `off <= self.buf.len()`
        unsafe { self.buf.get_unchecked_mut(self.off..) }
    }

    fn spare_len(&self) -> usize {
        self.buf.len() - self.off - self.len
    }

    fn spare_ptr(&mut self) -> *mut MaybeUninit<i32> {
        unsafe { self.buf.as_mut_ptr().add(self.off + self.len) }
    }
}

impl Cmsg {
    /// Write an fd.
    ///
    /// Returns `true` if there is remaining capacity. Otherwise, returns false and the fd is not
    /// written.
    #[inline]
    pub fn write_fd(&mut self, fd: i32) -> bool {
        let len = self.len;
        let Some(dst) = self.buf_mut().get_mut(len) else {
            return false;
        };
        dst.write(fd);
        self.len += 1;
        true
    }

    /// Read an fd.
    ///
    /// Returns `None` if there is no fd remaining.
    #[inline]
    pub fn read_fd(&mut self) -> Option<i32> {
        // idk why past me read from the back of the queue
        // let fd = *self.buf().get(self.len - 1)?;
        let fd = *self.buf().first()?;
        self.len -= 1;
        self.off += 1;
        // SAFETY: `len` represent count of initialized element
        Some(unsafe { fd.assume_init() })
    }

    /// Read `N` number of fds.
    ///
    /// Returns `None` if there is not enough fds remaining.
    #[inline]
    pub fn read_chunk<const N: usize>(&mut self) -> Option<[i32; N]> {
        let fd = self.buf().first_chunk::<N>()?.as_ptr();
        self.len -= N;
        self.off += N;
        // SAFETY: `len` represent count of initialized element
        // Some(unsafe { MaybeUninit::array_assume_init(fd) })
        Some(unsafe { *fd.cast::<[i32; N]>() })
    }

    /// Clear buffer.
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
        self.off = 0;
    }

    /// Perform `sendmsg` syscall.
    #[inline]
    pub fn sendmsg<S: AsFd>(
        &mut self,
        buf: &mut Bytes,
        socket: &S,
    ) -> Poll<Result<(), WriteError>> {
        sendmsg(buf, self, socket.as_fd().as_raw_fd())
    }

    /// Perform `recvmsg` syscall.
    #[inline]
    pub fn recvmsg<S: AsFd>(&mut self, buf: &mut Bytes, socket: &S) -> Poll<Result<(), ReadError>> {
        recvmsg(buf, self, socket.as_fd().as_raw_fd())
    }
}

// ===== CmsgBuf =====

const IMPOSTOR_CMSGHDR: usize = CMSGHDRSIZE / FDSIZE;

// - 1 for the actual header
// - the rest is actually represent fds
type CmsgBuf = [libc::cmsghdr; 1 + IMPOSTOR_CMSGHDR];

const NEW_CMSGHDR: libc::cmsghdr = libc::cmsghdr {
    cmsg_len: 0,
    cmsg_level: libc::SOL_SOCKET,
    cmsg_type: libc::SCM_RIGHTS,
};

const NEW_CMSGBUF: CmsgBuf = [NEW_CMSGHDR; _];

// this is the actual size/align when calculating using the macros
const CMSG_SPACE: usize = unsafe { libc::CMSG_SPACE(MAXCAP as u32) as usize };
const CMSG_ALIGN: usize = align_of::<libc::cmsghdr>();

// this prove all statements above
//
// perhaps this is not true for different architecture
const _: () = assert!(matches!(
    (size_of::<CmsgBuf>(), align_of::<CmsgBuf>()),
    (CMSG_SPACE, CMSG_ALIGN)
));
const _: () = unsafe { assert!(CMSGHDRSIZE == libc::CMSG_LEN(0) as usize) };

fn cmsg_len(fd_len: usize) -> usize {
    unsafe { libc::CMSG_LEN((fd_len * FDSIZE) as u32) as usize }
}

fn cmsg_space(fd_len: usize) -> usize {
    unsafe { libc::CMSG_SPACE((fd_len * FDSIZE) as u32) as usize }
}

fn sendmsg(buf: &mut Bytes, cmsg: &mut Cmsg, socket: i32) -> Poll<Result<(), WriteError>> {
    debug_assert!(!buf.is_empty());

    let mut cmsg_buf = NEW_CMSGBUF;
    let (msg_control, msg_controllen) = if cmsg.len == 0 {
        (ptr::null_mut(), 0)
    } else {
        cmsg_buf[0].cmsg_len = cmsg_len(cmsg.len);
        unsafe {
            // copy fds
            let src = cmsg.buf.as_ptr().add(cmsg.off);
            let dst = cmsg_buf.as_mut_ptr().add(1).cast();
            src.copy_to_nonoverlapping(dst, cmsg.len);
        }
        (cmsg_buf.as_mut_ptr().cast(), cmsg_space(cmsg.len))
    };
    let mut msghdr = libc::msghdr {
        msg_name: ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: &mut libc::iovec {
            iov_base: buf.as_mut_ptr().cast(),
            iov_len: buf.len(),
        },
        msg_iovlen: 1,
        msg_control,
        msg_controllen,
        msg_flags: 0,
    };

    loop {
        let Ok(write) = unsafe { libc::sendmsg(socket, &msghdr, 0) }.try_into() else {
            return ErrCode::would_block_or();
        };

        if write == buf.len() {
            buf.clear();
            cmsg.clear();
            break;
        }

        buf.advance(write);

        // update the advanced message buffer
        unsafe {
            let iovec = msghdr.msg_iov.as_mut_unchecked();
            iovec.iov_base = buf.as_mut_ptr().cast();
            iovec.iov_len = buf.len();
        }

        // Ancillary data is received as if it were queued along with the first normal data octet in
        // the segment (if any).
        //
        // https://unix.stackexchange.com/questions/185011/what-happens-with-unix-stream-ancillary-data-on-partial-reads
        //
        // unset the ancillary data
        msghdr.msg_control = ptr::null_mut();
        msghdr.msg_controllen = 0;
    }

    Poll::Ready(Ok(()))
}

fn recvmsg(buf: &mut Bytes, cmsg: &mut Cmsg, socket: i32) -> Poll<Result<(), ReadError>> {
    use ReadError as E;

    let mut cmsg_buf = NEW_CMSGBUF;
    let spare = buf.spare_capacity_mut();
    let mut msghdr = libc::msghdr {
        msg_name: ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: &mut libc::iovec {
            iov_base: spare.as_mut_ptr().cast(),
            iov_len: spare.len(),
        },
        msg_iovlen: 1,
        msg_control: cmsg_buf.as_mut_ptr().cast(),
        msg_controllen: CMSG_SPACE,
        msg_flags: 0,
    };

    let Ok(read) = unsafe { libc::recvmsg(socket, &mut msghdr, 0) }.try_into() else {
        return ErrCode::would_block_or_else(E::RecvErrno);
    };
    if read == 0 {
        debug_assert_ne!(buf.capacity(), 0);
        return Ready(Err(E::ConnectionAborted));
    }
    if msghdr.msg_flags & libc::MSG_CTRUNC == libc::MSG_CTRUNC {
        return Ready(Err(E::TruncatedControlData));
    }

    for cmsghdr in cmsghdrs(&msghdr) {
        if !cmsghdr.check_attrs() {
            return Ready(Err(E::InvalidControlData));
        }

        let data = cmsghdr.data();
        let len = cmsghdr.len();

        let fd_count = len / FDSIZE;
        let remaining = cmsg.spare_len();

        if fd_count > remaining {
            return Ready(Err(E::FdCapacityOverflow));
        }

        unsafe {
            cmsg.spare_ptr()
                .cast::<u8>()
                .copy_from_nonoverlapping(data, len);
        };
        cmsg.len += fd_count;

        debug_assert!(cmsg.len <= MAXFD);
    }

    unsafe {
        let new_len = buf.len().unchecked_add(read);
        buf.set_len(new_len);
    }

    Ready(Ok(()))
}

// ===== CmsgHdr =====

struct CmsgHdr<'a>(&'a libc::cmsghdr);

impl<'a> CmsgHdr<'a> {
    fn check_attrs(&self) -> bool {
        matches!(
            (self.0.cmsg_level, self.0.cmsg_type),
            (libc::SOL_SOCKET, libc::SCM_RIGHTS),
        )
    }

    fn data(&self) -> *mut u8 {
        unsafe { libc::CMSG_DATA(self.0) }
    }

    fn len(&self) -> usize {
        self.0.cmsg_len - CMSGHDRSIZE
    }
}

// ===== CmsgHdrIter =====

fn cmsghdrs(msghdr: &libc::msghdr) -> CmsgHdrIter<'_> {
    let cmsghdr = unsafe { libc::CMSG_FIRSTHDR(msghdr) };
    debug_assert!(cmsghdr.is_aligned());
    CmsgHdrIter {
        msghdr,
        cmsghdr: unsafe { cmsghdr.as_ref() },
    }
}

struct CmsgHdrIter<'a> {
    msghdr: &'a libc::msghdr,
    cmsghdr: Option<&'a libc::cmsghdr>,
}

impl<'a> Iterator for CmsgHdrIter<'a> {
    type Item = CmsgHdr<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let cmsghdr = self.cmsghdr.take()?;
        self.cmsghdr = unsafe { libc::CMSG_NXTHDR(self.msghdr, cmsghdr).as_ref() };
        Some(CmsgHdr(cmsghdr ))
    }
}

// ===== Error =====

#[derive(Clone, Copy)]
pub struct WriteError(ErrCode);

simple_os_error!(WriteError, "write to socket");

pub enum ReadError {
    RecvErrno(ErrCode),
    FdCapacityOverflow,
    InvalidControlData,
    TruncatedControlData,
    ConnectionAborted,
}

impl ReadError {
    #[inline]
    pub fn is_connection_aborted(&self) -> bool {
        matches!(self, Self::ConnectionAborted)
    }
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecvErrno(err) => write!(f, "failed to receive message: {err}"),
            Self::FdCapacityOverflow => write!(f, "fd capacity overflow"),
            Self::InvalidControlData => "unexpected ancillary data type".fmt(f),
            Self::TruncatedControlData => "ancillary data truncated".fmt(f),
            Self::ConnectionAborted => "connection aborted by the peer".fmt(f),
        }
    }
}
