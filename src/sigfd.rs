use std::io;
use std::mem::MaybeUninit;
use std::os::unix::io::{RawFd, AsRawFd, FromRawFd, OwnedFd};

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

// https://man7.org/linux/man-pages/man2/signalfd.2.html

pub struct Sigfd {
    fd: OwnedFd,
}

impl AsRawFd for Sigfd {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl Sigfd {
    pub fn new() -> io::Result<Self> {
        let mut mask = MaybeUninit::<libc::sigset_t>::uninit();
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

    pub fn read(&self) -> io::Result<()> {
        let mut fdsi = MaybeUninit::<libc::signalfd_siginfo>::uninit();
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
