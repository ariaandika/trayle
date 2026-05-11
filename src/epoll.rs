use std::ffi::c_int;
use std::io;
use std::mem::MaybeUninit;
use std::os::unix::io::{AsRawFd, RawFd};

use crate::macros::syscall;

#[derive(Clone, Copy)]
pub struct Interest(i32);

impl Interest {
    pub fn is_read(&self) -> bool {
        self.0 & libc::EPOLLIN == libc::EPOLLIN
    }

    pub fn is_write(&self) -> bool {
        self.0 & libc::EPOLLOUT == libc::EPOLLOUT
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

const EVENT_BUF_CAP: usize = 128;

pub struct EpollBuf([MaybeUninit<libc::epoll_event>; EVENT_BUF_CAP]);

impl EpollBuf {
    #[inline]
    pub fn new() -> Self {
        Self([MaybeUninit::uninit(); EVENT_BUF_CAP])
    }
}

pub struct Epoll {
    fd: c_int,
    len: u16,
    offset: u16,
}

impl Drop for Epoll {
    fn drop(&mut self) {
        if let Err(err) = syscall!(close, self.fd) {
            eprintln!("cannot close epoll fd: {err}");
        }
    }
}

impl Epoll {
    pub fn new() -> io::Result<Self> {
        let fd = syscall!(epoll_create1, 0)?;
        Ok(Self {
            fd,
            len: 0,
            offset: 0,
        })
    }

    pub fn add_read_interest<F: AsRawFd>(&self, key: u64, fd: &F) -> io::Result<()> {
        self.add_interest(key, (libc::EPOLLIN | libc::EPOLLRDHUP | libc::EPOLLET) as u32, fd.as_raw_fd())
    }

    // fn add_write_interest<F: AsRawFd>(&self, key: u64, fd: &F) -> io::Result<()> {
    //     self.add_interest(key, (libc::EPOLLOUT | libc::EPOLLET) as u32, fd.as_raw_fd())
    // }

    fn add_interest(&self, key: u64, events: u32, fd: RawFd) -> io::Result<()> {
        let event = libc::epoll_event { events, u64: key };
        syscall!(
            epoll_ctl,
            self.fd,
            libc::EPOLL_CTL_ADD,
            fd,
            std::ptr::from_ref(&event).cast_mut()
        )?;
        Ok(())
    }

    pub fn remove_interest<F: AsRawFd>(&self, fd: &F) -> io::Result<()> {
        syscall!(
            epoll_ctl,
            self.fd,
            libc::EPOLL_CTL_DEL,
            fd.as_raw_fd(),
            std::ptr::dangling_mut(),
        )?;
        Ok(())
    }

    /// Note that this will overwrite unread event.
    ///
    /// Should only be called if [`Epoll::next_event`] returns `None`,
    pub fn wait(&mut self, buf: &mut EpollBuf) -> io::Result<()> {
        self.len = 0;
        self.offset = 0;
        let result = syscall!(
            epoll_wait usize,
            self.fd,
            buf.0.as_mut_ptr().cast(),
            EVENT_BUF_CAP as i32,
            -1,
        );
        match result {
            Ok(nfds) => {
                self.len = nfds as u16;
                Ok(())
            }
            Err(err) => match err.kind() {
                io::ErrorKind::Interrupted => Ok(()),
                _ => Err(err),
            },
        }
    }

    pub fn next_event(&mut self, buf: &EpollBuf) -> Option<(u64, Interest)> {
        if self.len == self.offset {
            return None;
        }
        let event = unsafe { buf.0.get_unchecked(self.offset as usize).assume_init() };
        self.offset += 1;
        Some((event.u64, Interest(event.events as i32)))
    }
}

impl std::fmt::Debug for Interest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = std::fmt::from_fn(|f| {
            if self.is_read() {
                f.write_str("Read ")?;
            }
            if self.is_write() {
                f.write_str("Write ")?;
            }
            if self.is_close() {
                f.write_str("Close ")?;
            }
            f.write_fmt(format_args!("{:0>4b}", self.0))
        });
        f.debug_tuple("Interest").field(&msg).finish()
    }
}

