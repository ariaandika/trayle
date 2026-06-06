use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::ptr;

use crate::sys::errno::{Errno, simple_errno};

#[derive(Debug, Clone, Copy)]
pub struct Interest(i32);

impl Interest {
    pub fn is_write(&self) -> bool {
        self.0 & libc::EPOLLOUT == libc::EPOLLOUT
    }

    pub fn is_read(&self) -> bool {
        self.0 & libc::EPOLLIN == libc::EPOLLIN
    }

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
const DEFAULT_EVENT: i32 = libc::EPOLLIN | libc::EPOLLRDHUP | libc::EPOLLET;

impl Epoll {
    pub fn add<F: AsRawFd>(&self, key: u64, fd: &F) {
        self.epoll_ctl(libc::EPOLL_CTL_ADD, 0, key, fd.as_raw_fd())
    }

    pub fn modify<F: AsRawFd>(&self, is_write: bool, key: u64, fd: &F) {
        self.epoll_ctl(
            libc::EPOLL_CTL_MOD,
            libc::EPOLLOUT * is_write as i32,
            key,
            fd.as_raw_fd(),
        )
    }

    fn epoll_ctl(&self, op: i32, events: i32, key: u64, fd: RawFd) {
        let mut event = libc::epoll_event {
            events: (events | DEFAULT_EVENT) as u32,
            u64: key,
        };
        let result = unsafe { libc::epoll_ctl(self.0.as_raw_fd(), op, fd, &mut event) };
        if result == -1 {
            epoll_ctl_panic();
        }
    }

    pub fn delete<F: AsRawFd>(&self, fd: &F) {
        let result = unsafe {
            libc::epoll_ctl(
                self.0.as_raw_fd(),
                libc::EPOLL_CTL_DEL,
                fd.as_raw_fd(),
                ptr::null_mut(),
            )
        };
        if result == -1 {
            epoll_ctl_panic();
        }
    }
}

// ===== epoll_wait =====

impl Epoll {
    /// Waits for events and returns the number of events written to the given buffer.
    ///
    /// This method will block until either a file descriptor deliver an event, the call is
    /// interupted by a signal handler, or `timeout` expires.
    pub fn wait(&self, events: &mut [MaybeUninit<EpollEvent>], timeout: Option<u32>) -> usize {
        unsafe {
            let result = libc::epoll_wait(
                self.0.as_raw_fd(),
                events.as_mut_ptr().cast(),
                events.len() as i32,
                match timeout {
                    Some(ok) => (ok & i32::MAX as u32) as i32,
                    None => -1,
                },
            );
            match result.try_into() {
                Ok(nfds) => nfds,
                Err(_) => {
                    if *libc::__errno_location() != libc::EINTR {
                        epoll_wait_panic();
                    }
                    0
                }
            }
        }
    }
}

// ===== Errors =====

#[cold]
#[inline(never)]
fn epoll_ctl_panic() -> ! {
    // at some point, handling every epoll_ctl error become cumbersome, because it needs to sync
    // with the clients collection data structure
    //
    // looking further, all epoll_ctl failure is a server fault, except ENOMEM and ENOSPC, in which
    // no recovery seems possible, thus the panic
    panic!("`epoll_ctl` fail: {Errno}");
}

#[cold]
#[inline(never)]
fn epoll_wait_panic() -> ! {
    // all epoll_wait errors, except EINTR, are server error
    panic!("`epoll_wait` fail: {Errno}");
}

simple_errno! {
    pub CreateError, "failed to create epoll: {}";
}
