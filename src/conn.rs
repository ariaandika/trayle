use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, OwnedFd, RawFd};
use std::task::Poll;
use std::{ptr, slice};

use crate::buffer::Buffer;
use crate::errno::{Errno, simple_errno};
use crate::fd_buffer::FdBuffer;

#[derive(Debug)]
pub struct Connection(OwnedFd);

impl std::os::fd::FromRawFd for Connection {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        unsafe { Self(<_>::from_raw_fd(fd)) }
    }
}

impl AsRawFd for Connection {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl Connection {
    /// Read data to the read buffer.
    pub fn poll_read(&self, buffer: &mut Buffer, fds: &mut Buffer) -> Poll<Result<(), ReadError>> {
        recvmsg(buffer, fds, self.0.as_raw_fd())
    }

    /// Write data to the socket.
    ///
    /// May call write multiple time.
    pub fn poll_write_all(
        &self,
        buffer: &mut Buffer,
        fds: &mut FdBuffer,
    ) -> Poll<Result<(), WriteError>> {
        sendmsg(buffer, fds, self.0.as_raw_fd())
    }
}

// ===== syscall =====

fn sendmsg(buffer: &mut Buffer, fds: &mut FdBuffer, socket: RawFd) -> Poll<Result<(), WriteError>> {
    use libc::CMSG_FIRSTHDR;
    use libc::{SCM_RIGHTS, SOL_SOCKET, iovec, msghdr};

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
            iov_base: buffer.as_ptr() as *mut _,
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
        let Ok(write) = unsafe { libc::sendmsg(socket, &msghdr, 0) }.try_into() else {
            return match Errno::get() {
                libc::EWOULDBLOCK => Poll::Pending,
                _ => Poll::Ready(Err(WriteError)),
            };
        };
        buffer.advance(write);
        if buffer.is_empty() {
            break;
        }

        // rebuilt the message buffer
        let iov_mut = unsafe { &mut *msghdr.msg_iov };
        iov_mut.iov_base = buffer.as_ptr() as *mut _;
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

    fds.clear();

    Poll::Ready(Ok(()))
}

fn recvmsg(buffer: &mut Buffer, fds: &mut Buffer, socket: RawFd) -> Poll<Result<(), ReadError>> {
    use libc::{CMSG_DATA, CMSG_FIRSTHDR, CMSG_LEN, CMSG_NXTHDR};
    use libc::{SCM_RIGHTS, SOL_SOCKET, iovec, msghdr};
    use ReadError as E;

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

    let Ok(read) = unsafe { libc::recvmsg(socket, &mut msghdr, 0) }.try_into() else {
        return match Errno::get() {
            libc::EWOULDBLOCK => Poll::Pending,
            _ => Poll::Ready(Err(E::Errno)),
        };
    };
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

        buffer.advance_mut(read);
    }

    Poll::Ready(Ok(()))
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

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Errno => write!(f, "failed to read socket: {Errno}"),
            Self::ControlDataType => "unexpected ancillary data type".fmt(f),
            Self::ControlDataTruncated => "ancillary data truncated".fmt(f),
            Self::ConnectionAborted => std::io::ErrorKind::ConnectionAborted.fmt(f),
        }
    }
}
