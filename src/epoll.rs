use std::ffi::c_int;
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::{io, ptr};

#[derive(Debug, Clone, Copy)]
pub struct Interest(i32);

impl Interest {
    // pub fn is_read(&self) -> bool {
    //     self.0 & libc::EPOLLIN == libc::EPOLLIN
    // }
    //
    // pub fn is_write(&self) -> bool {
    //     self.0 & libc::EPOLLOUT == libc::EPOLLOUT
    // }
    //
    // pub fn is_shutdown(&self) -> bool {
    //     self.0 & libc::EPOLLRDHUP == libc::EPOLLRDHUP
    // }
    //
    // pub fn is_hangup(&self) -> bool {
    //     self.0 & libc::EPOLLHUP == libc::EPOLLHUP
    // }

    pub fn is_close(&self) -> bool {
        self.0 & (libc::EPOLLHUP | libc::EPOLLRDHUP) != 0
    }
}

// ===== EpollEvent =====

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct EpollEvent(libc::epoll_event);

impl EpollEvent {
    pub const fn key(&self) -> u64 {
        self.0.u64
    }

    pub const fn interest(&self) -> Interest {
        Interest(self.0.events as i32)
    }
}

// ===== Epoll =====

pub struct Epoll(OwnedFd);

impl Epoll {
    pub fn new() -> io::Result<Self> {
        let result = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };
        if result >= 0 {
            Ok(Self(unsafe { OwnedFd::from_raw_fd(result) }))
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}

// ===== epoll_ctl =====

/// peer closed connection, edge trigger notification.
const OTHER_EVENT: i32 = libc::EPOLLRDHUP | libc::EPOLLET;

impl Epoll {
    pub fn add_read<F: AsRawFd>(&self, key: u64, fd: &F) -> io::Result<()> {
        let mut event = libc::epoll_event {
            events: (libc::EPOLLIN | OTHER_EVENT) as u32,
            u64: key,
        };
        self.epoll_ctl(libc::EPOLL_CTL_ADD, fd.as_raw_fd(), &mut event)
    }

    // pub fn add_write<F: AsRawFd>(&self, key: u64, fd: &F) -> io::Result<()> {
    //     let mut event = libc::epoll_event {
    //         events: (libc::EPOLLOUT | OTHER_EVENT) as u32,
    //         u64: key,
    //     };
    //     self.epoll_ctl(libc::EPOLL_CTL_ADD, fd.as_raw_fd(), &mut event)
    // }

    pub fn remove<F: AsRawFd>(&self, fd: &F) -> io::Result<()> {
        self.epoll_ctl(libc::EPOLL_CTL_DEL, fd.as_raw_fd(), ptr::dangling_mut())
    }

    fn epoll_ctl(&self, op: c_int, fd: RawFd, event: *mut libc::epoll_event) -> io::Result<()> {
        let result = unsafe { libc::epoll_ctl(self.0.as_raw_fd(), op, fd, event) };
        if result == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

// ===== epoll_wait =====

impl Epoll {
    /// Waits for events and returns the number of events written to the given buffer.
    ///
    /// This method will block until either a file descriptor deliver an event, the call is
    /// interupted by a signal handler, or `timeout` expires.
    pub fn wait(
        &self,
        events: &mut [MaybeUninit<EpollEvent>],
        timeout: Option<u32>,
    ) -> io::Result<usize> {
        let result = unsafe {
            libc::epoll_wait(
                self.0.as_raw_fd(),
                events.as_mut_ptr().cast(),
                events.len() as i32,
                match timeout {
                    Some(ok) => (ok & i32::MAX as u32) as i32,
                    None => -1,
                },
            )
        };
        match usize::try_from(result) {
            Ok(nfds) => {
                unsafe { std::hint::assert_unchecked(nfds <= events.len()) };
                Ok(nfds)
            }
            Err(_) => {
                if result == libc::EINTR {
                    Ok(0)
                } else {
                    Err(io::Error::last_os_error())
                }
            }
        }
    }
}
