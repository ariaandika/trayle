use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, OwnedFd, RawFd};
use std::task::Poll;
use std::{ptr, slice};

use crate::buffer::Buffer;
use crate::errno::{self, ready};
use crate::fd_buffer::FdBuffer;

#[derive(Debug)]
pub struct Connection {
    fd: OwnedFd,
}

impl AsRawFd for Connection {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl Connection {
    pub const fn from_fd(fd: OwnedFd) -> Self {
        Self { fd }
    }

    /// Read data to the read buffer.
    pub fn poll_read(&self, buffer: &mut Buffer, fds: &mut Buffer) -> Poll<Result<(), MsgError>> {
        recvmsg(buffer, fds, self.fd.as_raw_fd())
    }

    /// Write data to the socket.
    ///
    /// May call write multiple time.
    pub fn poll_write_all(&self, buffer: &mut Buffer, fds: &mut FdBuffer) -> Poll<Result<(), MsgError>> {
        sendmsg(buffer, fds, self.fd.as_raw_fd())
    }
}

// ===== syscall =====

fn sendmsg(buffer: &mut Buffer, fds: &mut FdBuffer, socket: RawFd) -> Poll<Result<(), MsgError>> {
    use libc::CMSG_FIRSTHDR;
    use libc::{SCM_RIGHTS, SOL_SOCKET, c_void, iovec, msghdr};

    debug_assert!(!buffer.is_empty());

    let (cmsg_ptr, cmsg_len) = if fds.is_empty() {
        (ptr::null_mut(), 0)
    } else {
        let cmsg = fds.as_cmsg();

        debug_assert!(cmsg.len() as u32 == unsafe { libc::CMSG_LEN(fds.len() * size_of::<RawFd>() as u32) });

        (cmsg.as_ptr().cast_mut(), cmsg.len() as u32)
    };


    // https://linux.die.net/man/3/cmsg

    let msghdr = msghdr {
        msg_name: ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: &mut iovec {
            iov_base: buffer.as_ptr() as *mut c_void,
            iov_len: buffer.len(),
        },
        msg_iovlen: 1,
        msg_control: cmsg_ptr.cast(),
        msg_controllen: cmsg_len as usize,
        msg_flags: 0,
    };

    if !fds.is_empty() {
        let cmsg = unsafe { &mut *CMSG_FIRSTHDR(&msghdr) };
        cmsg.cmsg_len = cmsg_len as usize;
        cmsg.cmsg_level = SOL_SOCKET;
        cmsg.cmsg_type = SCM_RIGHTS;
        // payload initialized by FdBuffer
    }

    let mut msghdr = msghdr;
    loop {
        let write: usize = ready!(libc::sendmsg(socket, &msghdr, 0));
        if write == 0 {
            return Poll::Ready(Err(MsgError::WriteZero));
        }
        buffer.advance(write as u32);
        if buffer.is_empty() {
            break;
        }

        unsafe {
            // rebuilt the message buffer
            let iov_mut = &mut *msghdr.msg_iov;
            iov_mut.iov_base = buffer.as_ptr() as *mut c_void;
            iov_mut.iov_len = buffer.len();

            // Ancillary data is received as if it were queued along with the first normal data octet in
            // the segment (if any).
            //
            // - https://unix.stackexchange.com/questions/185011/what-happens-with-unix-stream-ancillary-data-on-partial-reads
            //
            // unset the ancillary data
            msghdr.msg_control = ptr::null_mut();
            msghdr.msg_controllen = 0;
        }
    }

    fds.clear();

    Poll::Ready(Ok(()))
}

fn recvmsg(buffer: &mut Buffer, fds: &mut Buffer, socket: RawFd) -> Poll<Result<(), MsgError>> {
    use libc::{CMSG_DATA, CMSG_FIRSTHDR, CMSG_LEN, CMSG_NXTHDR};
    use libc::{SCM_RIGHTS, SOL_SOCKET, iovec, msghdr};
    use MsgError as E;

    const CMSG_SPACE: u32 = unsafe { libc::CMSG_SPACE(crate::MAX_FD_SIZE) };

    let spare_buf = buffer.spare_capacity_mut();

    let mut cmsg_buf = [const { MaybeUninit::<u8>::uninit() }; CMSG_SPACE as usize];
    let mut iov = iovec {
        iov_base: spare_buf.as_mut_ptr().cast(),
        iov_len: spare_buf.len(),
    };
    let mut msghdr = msghdr {
        msg_name: ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: &mut iov,
        msg_iovlen: 1,
        msg_control: cmsg_buf.as_mut_ptr().cast(),
        msg_controllen: cmsg_buf.len(),
        msg_flags: 0,
    };

    let read: usize = ready!(libc::recvmsg(socket, &mut msghdr, 0));
    if read == 0 {
        return Poll::Ready(Err(E::ConnectionAborted));
    }
    if msghdr.msg_flags & libc::MSG_CTRUNC == libc::MSG_CTRUNC {
        return Poll::Ready(Err(E::ControlDataTruncated));
    }

    unsafe {
        let mut cmsg_ptr = CMSG_FIRSTHDR(&msghdr);
        while !cmsg_ptr.is_null() {
            let cmsg = cmsg_ptr.read_unaligned();

            let (SOL_SOCKET, SCM_RIGHTS) = (cmsg.cmsg_level, cmsg.cmsg_type) else {
                return Poll::Ready(Err(E::ControlDataType));
            };

            let bytes_len = cmsg.cmsg_len - const { CMSG_LEN(0) as usize };
            let bytes = slice::from_raw_parts(CMSG_DATA(&cmsg), bytes_len);

            fds.extend_from_slice(bytes);

            cmsg_ptr = CMSG_NXTHDR(&msghdr, cmsg_ptr);
        }

        buffer.advance_mut(read as u32);
    }

    Poll::Ready(Ok(()))
}

// ===== MsgError =====

pub enum MsgError {
    Errno,
    WriteZero,
    ControlDataType,
    ControlDataTruncated,
    ConnectionAborted,
}

impl From<errno::Errno> for MsgError {
    fn from(_: errno::Errno) -> Self {
        Self::Errno
    }
}

impl std::fmt::Display for MsgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Errno => std::io::Error::last_os_error().fmt(f),
            Self::WriteZero => std::io::ErrorKind::WriteZero.fmt(f),
            Self::ControlDataType => "unexpected ancillary data type".fmt(f),
            Self::ControlDataTruncated => "ancillary data truncated".fmt(f),
            Self::ConnectionAborted => std::io::ErrorKind::ConnectionAborted.fmt(f),
        }
    }
}
