use std::io;
use std::os::fd::RawFd;
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::net::{UnixListener, UnixStream};

pub struct EventLoop {
    epoll: Epoll,
    sigfd: Sigfd,
    listener: UnixListener,
    streams: Vec<UnixStream>,
}

#[derive(Debug)]
pub enum EventKind<'a> {
    New(&'a mut UnixStream),
    ReadWrite(&'a mut UnixStream, Interest),
    Close(UnixStream, Interest),
}

const LISTENER_KEY: u64 = 0;
const SIGFD_KEY: u64 = 1;
const KEY_OFFSET: u64 = 2;

impl EventLoop {
    pub fn new(path: &str) -> io::Result<Self> {
        let listener = UnixListener::bind(path)?;
        listener.set_nonblocking(true)?;

        let sigfd = Sigfd::new()?;

        let epoll = Epoll::new()?;
        epoll.add_read_interest(LISTENER_KEY, &listener)?;
        epoll.add_read_interest(SIGFD_KEY, &sigfd.fd)?;

        Ok(Self {
            epoll,
            sigfd,
            listener,
            streams: Vec::with_capacity(32),
        })
    }
}

impl EventLoop {
    /// Get the next event.
    ///
    /// This will block current thread.
    pub fn next_event(&mut self) -> io::Result<Option<EventKind<'_>>> {
        loop {
            let Some((key, interest)) = self.epoll.next_event() else {
                eprintln!("[EPOLL] blocking");
                self.epoll.wait()?;
                eprintln!("[EPOLL] complete");
                continue;
            };
            match key {
                LISTENER_KEY => {
                    let stream = match self.listener.accept() {
                        Ok((stream, _)) => stream,
                        Err(err) => match err.kind() {
                            io::ErrorKind::WouldBlock => continue,
                            _ => return Err(err),
                        },
                    };

                    stream.set_nonblocking(true)?;
                    self.epoll.add_read_interest(self.streams.len() as u64 + KEY_OFFSET, &stream)?;
                    let stream = self.streams.push_mut(stream);

                    break Ok(Some(EventKind::New(stream)));
                },
                SIGFD_KEY => {
                    self.sigfd.read()?;
                    return Ok(None);
                }
                key => {
                    if interest.is_close() {
                        let stream = self.streams.swap_remove((key - KEY_OFFSET) as usize);
                        self.epoll.remove_interest(&stream)?;
                        break Ok(Some(EventKind::Close(stream, interest)));
                    } else {
                        let stream = &mut self.streams[(key - KEY_OFFSET) as usize];
                        break Ok(Some(EventKind::ReadWrite(stream, interest)));
                    }
                }
            }
        }
    }
}

// ===== epoll =====

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

struct Epoll {
    fd: OwnedFd,
    events: Vec<libc::epoll_event>,
    offset: usize,
}

impl Epoll {
    fn new() -> io::Result<Self> {
        unsafe {
            let fd = syscall!(epoll_create1, 0)?;
            Ok(Self {
                fd: OwnedFd::from_raw_fd(fd),
                events: Vec::with_capacity(32),
                offset: 0,
            })
        }
    }

    fn add_read_interest<F: AsRawFd>(&self, key: u64, fd: &F) -> io::Result<()> {
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

    fn remove_interest<F: AsRawFd>(&self, fd: &F) -> io::Result<()> {
        syscall!(
            epoll_ctl,
            self.fd.as_raw_fd(),
            libc::EPOLL_CTL_DEL,
            fd.as_raw_fd(),
            std::ptr::dangling_mut(),
        )?;
        Ok(())
    }

    fn wait(&mut self) -> io::Result<()> {
        let spare = self.events.spare_capacity_mut();
        let nfds = syscall!(
            usize,
            epoll_wait,
            self.fd.as_raw_fd(),
            spare.as_mut_ptr().cast(),
            spare.len() as i32,
            -1,
        )?;
        unsafe { self.events.set_len(self.events.len() + nfds) };
        Ok(())
    }

    fn next_event(&mut self) -> Option<(u64, Interest)> {
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

// ===== sigfd =====

// https://man7.org/linux/man-pages/man2/signalfd.2.html

struct Sigfd {
    fd: OwnedFd,
}

impl Sigfd {
    fn new() -> io::Result<Self> {
        let mut mask = std::mem::MaybeUninit::<libc::sigset_t>::uninit();
        syscall!(sigemptyset, mask.as_mut_ptr())?;
        syscall!(sigaddset, mask.as_mut_ptr(), libc::SIGINT)?;
        syscall!(
            sigprocmask,
            libc::SIG_BLOCK,
            mask.as_ptr(),
            std::ptr::null_mut()
        )?;

        let fd = syscall!(signalfd, -1, mask.as_ptr(), libc::SFD_NONBLOCK)?;

        Ok(Self {
            fd: unsafe { OwnedFd::from_raw_fd(fd) },
        })
    }

    fn read(&self) -> io::Result<()> {
        let mut fdsi = std::mem::MaybeUninit::<libc::signalfd_siginfo>::uninit();
        let len = syscall!(
            usize,
            read,
            self.fd.as_raw_fd(),
            fdsi.as_mut_ptr().cast(),
            size_of::<libc::signalfd_siginfo>()
        )?;
        if len == size_of::<libc::signalfd_siginfo>() {
            let fdsi = unsafe { fdsi.assume_init() };
            if fdsi.ssi_signo != libc::SIGINT as u32 {
                eprintln!("`sigfd` returns unhandled signal: `{}`", fdsi.ssi_signo);
            }
        }
        Ok(())
    }
}
