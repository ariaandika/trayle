use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::ptr;

use crate::errno::simple_errno;

#[derive(Debug, Clone, Copy)]
pub struct Interest(i32);

impl Interest {
    pub fn is_write(&self) -> bool {
        self.0 & libc::EPOLLOUT == libc::EPOLLOUT
    }

    // pub fn is_read(&self) -> bool {
    //     self.0 & libc::EPOLLIN == libc::EPOLLIN
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

#[repr(transparent)]
pub struct EpollEvent(libc::epoll_event);

impl EpollEvent {
    /// Returns `(key, Interest)`.
    pub fn to_parts(&self) -> (u64, Interest) {
        (self.0.u64, Interest(self.0.events as i32))
    }
}

// ===== Epoll =====

pub struct Epoll(OwnedFd);

impl Epoll {
    pub fn new() -> Result<Self, CreateError> {
        unsafe {
            let fd = libc::epoll_create1(libc::EPOLL_CLOEXEC);
            if fd == -1 {
                return Err(CreateError);
            }
            Ok(Self(<_>::from_raw_fd(fd)))
        }
    }
}

// ===== epoll_ctl =====

/// peer closed connection, edge trigger notification.
const OTHER_EVENT: i32 = libc::EPOLLRDHUP | libc::EPOLLET;

impl Epoll {
    pub fn add_read<F: AsRawFd>(&self, key: u64, fd: &F) -> Result<(), AddError> {
        let mut event = libc::epoll_event {
            events: (libc::EPOLLIN | OTHER_EVENT) as u32,
            u64: key,
        };
        match self.epoll_ctl(libc::EPOLL_CTL_ADD, fd.as_raw_fd(), &mut event) {
            0 => Ok(()),
            _ => Err(AddError),
        }
    }

    // pub fn add_write<F: AsRawFd>(&self, key: u64, fd: &F) -> Result<()> {
    //     let mut event = libc::epoll_event {
    //         events: (libc::EPOLLOUT | OTHER_EVENT) as u32,
    //         u64: key,
    //     };
    //     self.epoll_ctl(libc::EPOLL_CTL_ADD, fd.as_raw_fd(), &mut event)
    // }

    pub fn remove<F: AsRawFd>(&self, fd: &F) -> Result<(), RemoveError> {
        match self.epoll_ctl(libc::EPOLL_CTL_DEL, fd.as_raw_fd(), ptr::null_mut()) {
            0 => Ok(()),
            _ => Err(RemoveError),
        }
    }

    fn epoll_ctl(&self, op: i32, fd: RawFd, event: *mut libc::epoll_event) -> i32 {
        unsafe { libc::epoll_ctl(self.0.as_raw_fd(), op, fd, event) }
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
    ) -> Result<usize, WaitError> {
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
        match result.try_into() {
            Ok(nfds) => Ok(nfds),
            Err(_) => unsafe {
                if *libc::__errno_location() == libc::EINTR {
                    Ok(0)
                } else {
                    Err(WaitError)
                }
            },
        }
    }
}

// ===== Errors =====

// `man 2 epoll_create1`
// EINVAL size is not positive.
// EINVAL (epoll_create1()) Invalid value specified in flags.
// EMFILE The per-process limit on the number of open file descriptors has been reached.
// ENFILE The system-wide limit on the total number of open files has been reached.
// ENOMEM There was insufficient memory to create the kernel object.

simple_errno! {
    pub CreateError, "failed to create epoll: {}";
    pub AddError, "failed to add epoll fd: {}";
    pub RemoveError, "failed to remove epoll fd: {}";
    pub WaitError, "failed to wait for epoll event: {}";
}
