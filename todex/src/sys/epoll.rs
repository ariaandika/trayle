use std::mem::MaybeUninit;
use std::os::fd::*;
use std::ptr;

use crate::sys::error::{ErrCode, simple_os_error};

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

impl FromRawFd for Epoll {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self(unsafe { <_>::from_raw_fd(fd) })
    }
}

impl AsFd for Epoll {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl Epoll {
    #[inline]
    pub fn new() -> Result<Self, CreateError> {
        ErrCode::from_raw_fd(unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) })
    }
}

// ===== epoll_ctl =====

/// peer closed connection, edge trigger notification.
const DEFAULT_EVENT: i32 = libc::EPOLLIN | libc::EPOLLRDHUP | libc::EPOLLET;

impl Epoll {
    pub fn add<F: AsFd>(&self, key: u64, fd: &F) {
        self.epoll_ctl(libc::EPOLL_CTL_ADD, 0, key, fd.as_fd())
    }

    pub fn modify<F: AsFd>(&self, is_write: bool, key: u64, fd: &F) {
        self.epoll_ctl(
            libc::EPOLL_CTL_MOD,
            libc::EPOLLOUT * is_write as i32,
            key,
            fd.as_fd(),
        )
    }

    fn epoll_ctl(&self, op: i32, events: i32, key: u64, fd: BorrowedFd) {
        let mut event = libc::epoll_event {
            events: (events | DEFAULT_EVENT) as u32,
            u64: key,
        };
        let result = unsafe { libc::epoll_ctl(self.0.as_raw_fd(), op, fd.as_raw_fd(), &mut event) };
        if result == -1 {
            epoll_ctl_panic();
        }
    }

    pub fn delete<F: AsFd>(&self, fd: &F) {
        let result = unsafe {
            libc::epoll_ctl(
                self.0.as_raw_fd(),
                libc::EPOLL_CTL_DEL,
                fd.as_fd().as_raw_fd(),
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
    #[inline]
    pub fn wait(&self, events: &mut [MaybeUninit<EpollEvent>], timeout: Option<u32>) -> usize {
        unsafe {
            libc::epoll_wait(
                self.0.as_raw_fd(),
                events.as_mut_ptr().cast(),
                events.len() as i32,
                timeout.map_or(-1, |t| t as i32),
            )
            .try_into()
            .inspect_err(|_| {
                if *libc::__errno_location() != libc::EINTR {
                    epoll_wait_panic();
                }
            })
            .unwrap_or(0)
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
    panic!("`epoll_ctl` fail: {}", ErrCode::errno());
}

#[cold]
#[inline(never)]
fn epoll_wait_panic() -> ! {
    // all epoll_wait errors, except EINTR, are server error
    panic!("`epoll_wait` fail: {}", ErrCode::errno());
}

#[derive(Clone, Copy)]
pub struct CreateError(ErrCode);

simple_os_error!(CreateError, "create epoll");
