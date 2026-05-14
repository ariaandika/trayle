use std::io;
use std::mem;
use std::os::fd::AsRawFd;
use std::task::Poll;

use crate::macros::syscall;
use crate::conn::Connection;

// ===== SocketPath =====

pub struct SocketPath {
    path: *const std::os::raw::c_char,
    addr: libc::sockaddr_un,
    len: libc::socklen_t,
}

impl SocketPath {
    pub const fn new(path: &'static std::ffi::CStr) -> Self {
        // SAFETY: All zeros is a valid representation for `sockaddr_un`.
        let mut addr: libc::sockaddr_un = unsafe { mem::zeroed() };

        // https://man7.org/linux/man-pages/man7/unix.7.html
        addr.sun_family = libc::AF_UNIX as libc::sa_family_t;

        let bytes = path.to_bytes_with_nul();

        assert!(bytes.len() < addr.sun_path.len());

        // SAFETY: `bytes` and `addr.sun_path` are not overlapping and both point to valid memory.
        unsafe {
            addr.sun_path
                .as_mut_ptr()
                .copy_from_nonoverlapping(bytes.as_ptr().cast(), bytes.len());
        };

        const SUN_PATH_OFFSET: usize = mem::offset_of!(libc::sockaddr_un, sun_path);

        let mut len = (SUN_PATH_OFFSET + bytes.len()) as libc::socklen_t;
        match bytes.first() {
            Some(&0) | None => {}
            Some(_) => len += 1,
        }
        let path = path.as_ptr();
        Self { path, addr, len }
    }
}

// ===== Listener =====

pub struct Listener {
    fd: i32,
    path: *const std::os::raw::c_char,
}

impl Drop for Listener {
    fn drop(&mut self) {
        if let Err(err) = syscall!(close, self.fd) {
            eprintln!("cannot close socket: {err}");
        }
        if let Err(err) = syscall!(unlink, self.path) {
            eprintln!("cannot unlink socket: {err}");
        }
    }
}

impl Listener {
    pub fn new(path: SocketPath) -> io::Result<Self> {
        let fd = syscall!(socket, libc::AF_UNIX, libc::SOCK_STREAM, 0)?;
        let SocketPath { addr, len, path } = path;

        /// https://man7.org/linux/man-pages/man2/listen.2.html#NOTES
        const BACKLOG: i32 = -1;

        syscall!(bind, fd, (&raw const addr) as *const _, len)?;
        syscall!(listen, fd, BACKLOG)?;

        // there is also using `fnctl`, but its about standard thing
        syscall!(ioctl, fd, libc::FIONBIO, &mut true)?;

        Ok(Self { fd, path })
    }

    pub fn poll_accept(&self) -> Poll<io::Result<Connection>> {
        match syscall!(accept, self.fd, std::ptr::null_mut(), std::ptr::null_mut()) {
            Ok(fd) => {
                syscall!(ioctl, fd, libc::FIONBIO, &mut true)?;
                let Some(fd) = std::num::NonZeroI32::new(fd) else {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "zero value fd",
                    )));
                };
                Poll::Ready(Ok(Connection::from_fd(fd)))
            }
            Err(err) => match err.kind() {
                io::ErrorKind::WouldBlock => Poll::Pending,
                _ => Poll::Ready(Err(err)),
            },
        }
    }
}

impl AsRawFd for Listener {
    fn as_raw_fd(&self) -> i32 {
        self.fd
    }
}
