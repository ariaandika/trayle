use std::io;
use std::os::fd::RawFd;
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd};

macro_rules! syscall {
    (usize, $f:ident, $($tt:tt)*) => {
        {
            let result = unsafe { libc::$f($($tt)*) };
            match usize::try_from(result) {
                Ok(ok) => Ok(ok),
                Err(_) => Err(io::Error::last_os_error()),
            }
        }
    };
    ($f:ident, $($tt:tt)*) => {
        {
            #[allow(unused_unsafe)]
            let result = unsafe { libc::$f($($tt)*) };
            if result >= 0 {
                Ok(result)
            } else {
                Err(io::Error::last_os_error())
            }
        }
    };
}

#[derive(Clone, Copy)]
pub struct Interest(i32);

impl Interest {
    pub fn is_read(&self) -> bool {
        self.0 & libc::EPOLLIN == libc::EPOLLIN
    }

    pub fn is_write(&self) -> bool {
        self.0 & libc::EPOLLOUT == libc::EPOLLOUT
    }

    pub fn is_shutdown(&self) -> bool {
        self.0 & libc::EPOLLRDHUP == libc::EPOLLRDHUP
    }

    pub fn is_hangup(&self) -> bool {
        self.0 & libc::EPOLLHUP == libc::EPOLLHUP
    }

    pub fn is_close(&self) -> bool {
        self.0 & (libc::EPOLLHUP | libc::EPOLLRDHUP) != 0
    }
}

pub struct Epoll {
    fd: OwnedFd,
    events: Vec<libc::epoll_event>,
    offset: usize,
}

impl Epoll {
    pub fn new() -> io::Result<Self> {
        unsafe {
            let fd = syscall!(epoll_create1, 0)?;
            Ok(Self {
                fd: OwnedFd::from_raw_fd(fd),
                events: Vec::with_capacity(32),
                offset: 0,
            })
        }
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
            self.fd.as_raw_fd(),
            libc::EPOLL_CTL_ADD,
            fd,
            std::ptr::from_ref(&event).cast_mut()
        )?;
        Ok(())
    }

    pub fn remove_interest<F: AsRawFd>(&self, fd: &F) -> io::Result<()> {
        syscall!(
            epoll_ctl,
            self.fd.as_raw_fd(),
            libc::EPOLL_CTL_DEL,
            fd.as_raw_fd(),
            std::ptr::dangling_mut(),
        )?;
        Ok(())
    }

    pub fn wait(&mut self) -> io::Result<()> {
        let spare = self.events.spare_capacity_mut();
        let result = unsafe {
            libc::epoll_wait(
                self.fd.as_raw_fd(),
                spare.as_mut_ptr().cast(),
                spare.len() as i32,
                -1,
            )
        };
        match usize::try_from(result) {
            Ok(nfds) => unsafe {
                self.events.set_len(self.events.len() + nfds);
                Ok(())
            },
            Err(_) => match result {
                libc::EINTR => Ok(()),
                _ => Err(io::Error::last_os_error()),
            },
        }
    }

    pub fn next_event(&mut self) -> Option<(u64, Interest)> {
        let Some(event) = self.events.get(self.offset) else {
            unsafe { self.events.set_len(0) };
            self.offset = 0;
            return None;
        };
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

