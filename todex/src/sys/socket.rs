use std::ffi::CStr;
use std::mem;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::ptr::NonNull;

use crate::sys::errno::Errno;

pub struct Socket {
    fd: OwnedFd,
}

impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> i32 {
        self.fd.as_raw_fd()
    }
}

impl FromRawFd for Socket {
    unsafe fn from_raw_fd(fd: i32) -> Self {
        Self {
            fd: unsafe { <_>::from_raw_fd(fd) },
        }
    }
}

impl Socket {
    /// Connect from `WAYLAND_DISPLAY`.
    pub fn connect_env() -> Result<Self, ConnectError> {
        use ConnectError as E;

        let addr = sockaddr_un()?;
        let sockaddr = (&raw const addr).cast();

        let socket = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0) };
        if socket == -1 {
            return Err(E::Socket);
        }

        let result = unsafe { libc::connect(socket, sockaddr, size_of_val(&addr) as libc::socklen_t) };
        if result == -1 {
            return Err(E::Connect);
        }

        let fd = unsafe { OwnedFd::from_raw_fd(socket) };

        Ok(Self { fd })
    }
}

fn env(name: &CStr) -> Option<&CStr> {
    NonNull::new(unsafe { libc::getenv(name.as_ptr()) })
        .map(|e| unsafe { CStr::from_ptr(e.as_ptr()) })
        .filter(|e| !e.is_empty())
}

/// `man 7 unix`
fn sockaddr_un() -> Result<libc::sockaddr_un, ConnectError> {
    use ConnectError as E;

    // SAFETY: all zeros value is valid representation for `sockaddr_un`
    let mut addr: libc::sockaddr_un = unsafe { mem::zeroed() };
    // The sun_family field always contains AF_UNIX
    addr.sun_family = libc::AF_UNIX as u16;

    let mut path = unsafe { mem::transmute::<&mut [i8], &mut [u8]>(addr.sun_path.as_mut_slice()) };

    let display = env(c"WAYLAND_DISPLAY")
        .unwrap_or(c"wayland-0")
        .to_bytes_with_nul();

    if display[0] != b'/' {
        let runtime = env(c"XDG_RUNTIME_DIR")
            .unwrap_or(c"/run/user/1000")
            .to_bytes();
        let Some((dst, rest)) = path.split_at_mut_checked(runtime.len()) else {
            return Err(E::AddrTooLong);
        };
        dst.copy_from_slice(runtime);
        path = match runtime.last() {
            Some(b'/') | None => rest,
            Some(_) => {
                let Some((lead, rest)) = rest.split_first_mut() else {
                    return Err(E::AddrTooLong);
                };
                *lead = b'/';
                rest
            },
        };
    }

    let Some(dst) = path.get_mut(..display.len()) else {
        return Err(E::AddrTooLong);
    };
    dst.copy_from_slice(display);
    Ok(addr)
}

impl std::fmt::Debug for Socket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Socket")
    }
}

// ===== Error =====

#[derive(Debug)]
pub enum ConnectError {
    AddrTooLong,
    Socket,
    Connect,
}

impl std::error::Error for ConnectError {}

impl std::fmt::Display for ConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddrTooLong => write!(f, "address too long"),
            Self::Socket => write!(f, "failed to create socket: {Errno}"),
            Self::Connect => write!(f, "failed to connect socket: {Errno}"),
        }
    }
}
