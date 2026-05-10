use std::io;
use std::os::fd::RawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd};

pub struct EventLoop {
    epoll: Epoll,
    sigfd: Sigfd,
    listener: UnixListener,
    streams: Vec<UnixStream>,
}

pub enum EventKind<'a> {
    Incoming(&'a mut UnixStream),
    Sigint,
}

const LISTENER_KEY: u64 = 0;
const SIGFD_KEY: u64 = 1;

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
    /// Wait for any io events.
    ///
    /// This will block current thread.
    pub fn wait_events(&mut self) -> io::Result<()> {
        self.epoll.wait()
    }

    pub fn next_event(&mut self) -> io::Result<Option<EventKind<'_>>> {
        loop {
            let Some(event_key) = self.epoll.next_event_key() else {
                return Ok(None);
            };
            match event_key {
                LISTENER_KEY => {
                    let stream = match self.listener.accept() {
                        Ok((stream, _)) => stream,
                        Err(err) => match err.kind() {
                            io::ErrorKind::WouldBlock => continue,
                            _ => return Err(err),
                        },
                    };

                    stream.set_nonblocking(true)?;
                    self.epoll.add_read_interest(self.streams.len() as u64, &stream)?;
                    let stream = self.streams.push_mut(stream);

                    break Ok(Some(EventKind::Incoming(stream)));
                },
                SIGFD_KEY => {
                    self.sigfd.read()?;
                    break Ok(Some(EventKind::Sigint));
                }
                _ => todo!(),
            }
        }
    }
}

// ===== epoll =====

struct Epoll {
    fd: OwnedFd,
    events: Vec<libc::epoll_event>,
    offset: usize,
}

impl Epoll {
    fn new() -> io::Result<Self> {
        unsafe {
            let fd = libc::epoll_create1(0);
            if fd.is_negative() {
                return Err(io::Error::last_os_error());
            }
            Ok(Self {
                fd: OwnedFd::from_raw_fd(fd),
                events: Vec::with_capacity(32),
                offset: 0,
            })
        }
    }

    fn add_read_interest<F: AsRawFd>(&self, key: u64, fd: &F) -> io::Result<()> {
        self.add_interest(key, (libc::EPOLLIN | libc::EPOLLET) as u32, fd.as_raw_fd())
    }

    // fn add_write_interest<F: AsRawFd>(&self, key: u64, fd: &F) -> io::Result<()> {
    //     self.add_interest(key, (libc::EPOLLOUT | libc::EPOLLET) as u32, fd.as_raw_fd())
    // }

    fn add_interest(&self, key: u64, events: u32, fd: RawFd) -> io::Result<()> {
        const OP: i32 = libc::EPOLL_CTL_ADD;
        let event = libc::epoll_event { events, u64: key };
        let result = unsafe {
            libc::epoll_ctl(
                self.fd.as_raw_fd(),
                OP,
                fd,
                std::ptr::from_ref(&event).cast_mut(),
            )
        };
        match usize::try_from(result) {
            Ok(_) => Ok(()),
            Err(_) => Err(io::Error::last_os_error()),
        }
    }

    fn wait(&mut self) -> io::Result<()> {
        const TIMEOUT: std::ffi::c_int = -1;

        let spare = self.events.spare_capacity_mut();
        let nfds = unsafe {
            libc::epoll_wait(
                self.fd.as_raw_fd(),
                spare.as_mut_ptr().cast(),
                spare.len() as i32,
                TIMEOUT,
            )
        };
        let Ok(nfds) = usize::try_from(nfds) else {
            return Err(io::Error::last_os_error());
        };
        unsafe { self.events.set_len(self.events.len() + nfds) };
        Ok(())
    }

    fn next_event_key(&mut self) -> Option<u64> {
        let Some(event) = self.events.get(self.offset) else {
            unsafe { self.events.set_len(0) };
            self.offset = 0;
            return None;
        };
        self.offset += 1;
        Some(event.u64)
    }
}

// ===== sigfd =====

struct Sigfd {
    fd: OwnedFd,
}

impl Sigfd {
    fn new() -> io::Result<Self> {
        unsafe {
            let mut mask = std::mem::MaybeUninit::<libc::sigset_t>::uninit();

            libc::sigemptyset(mask.as_mut_ptr());
            libc::sigaddset(mask.as_mut_ptr(), libc::SIGINT);

            let result = libc::sigprocmask(libc::SIG_BLOCK, mask.as_ptr(), std::ptr::null_mut());
            if result.is_negative() {
                return Err(io::Error::last_os_error());
            }

            let fd = libc::signalfd(-1, mask.as_ptr(), libc::SFD_NONBLOCK);
            if fd.is_negative() {
                return Err(io::Error::last_os_error());
            }

            Ok(Self {
                fd: OwnedFd::from_raw_fd(fd),
            })
        }
    }

    fn read(&self) -> io::Result<()> {
        unsafe {
            let mut fdsi = std::mem::MaybeUninit::<libc::signalfd_siginfo>::uninit();

            let len = libc::read(
                self.fd.as_raw_fd(),
                fdsi.as_mut_ptr().cast(),
                size_of::<libc::signalfd_siginfo>(),
            );
            let Ok(len) = usize::try_from(len) else {
                return Err(io::Error::last_os_error());
            };
            debug_assert_eq!(len, size_of::<libc::signalfd_siginfo>());

            let fdsi = fdsi.assume_init();
            if fdsi.ssi_signo != libc::SIGINT as u32 {
                eprintln!("`sigfd` returns unhandled signal: `{}`", fdsi.ssi_signo);
            }
        }
        Ok(())
    }
}
