use std::os::fd::{AsRawFd, OwnedFd, RawFd};
use std::task::Poll;

use crate::buffer::Buffer;
use crate::errno::{Errno, simple_errno};

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
    pub fn poll_read(&self, buffer: &mut Buffer) -> Poll<Result<(), ReadError>> {
        recvmsg(buffer, self.0.as_raw_fd())
    }

    /// Write all data to the socket.
    ///
    /// If write is pending, add write event to epoll.
    pub fn poll_write_all(&self, buffer: &mut Buffer) -> Poll<Result<(), WriteError>> {
        sendmsg(buffer, self.0.as_raw_fd())
    }
}

// ===== syscall =====

fn sendmsg(buffer: &mut Buffer, socket: RawFd) -> Poll<Result<(), WriteError>> {
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

fn recvmsg(buffer: &mut Buffer, socket: RawFd) -> Poll<Result<(), ReadError>> {
    use ReadError as E;

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
        Poll::Ready(Ok(()))
    } else {
        Poll::Ready(Err(E::ControlDataType))
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
