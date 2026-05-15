use std::io;
use std::mem::MaybeUninit;
use std::os::unix::io::{AsRawFd, RawFd};

use crate::macros::syscall;

// https://man7.org/linux/man-pages/man2/signalfd.2.html

pub enum Sig {
    Int,
    Term,
}

impl Sig {
    const SIGNALS: [i32; 2] = [libc::SIGINT, libc::SIGTERM];

    fn from_sig(sig: i32) -> Option<Sig> {
        match sig {
            libc::SIGINT => Some(Self::Int),
            libc::SIGTERM => Some(Self::Term),
            _ => None,
        }
    }
}

impl std::fmt::Display for Sig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sig::Int => "SIGINT",
            Sig::Term => "SIGTERM",
        }
        .fmt(f)
    }
}

pub struct Sigfd {
    fd: i32,
}

impl Drop for Sigfd {
    fn drop(&mut self) {
        if let Err(err) = syscall!(close, self.fd) {
            eprintln!("cannot close sigfd fd: {err}");
        }
    }
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
        for signal in Sig::SIGNALS {
            syscall!(sigaddset, mask.as_mut_ptr(), signal)?;
        }
        syscall!(
            sigprocmask,
            libc::SIG_BLOCK,
            mask.as_ptr(),
            std::ptr::null_mut()
        )?;

        let fd = syscall!(signalfd, -1, mask.as_ptr(), libc::SFD_NONBLOCK)?;

        Ok(Self { fd })
    }

    pub fn read(&self) -> io::Result<Option<Sig>> {
        let mut fdsi = MaybeUninit::<libc::signalfd_siginfo>::uninit();
        let len = syscall!(read(
            self.fd.as_raw_fd(),
            fdsi.as_mut_ptr().cast(),
            size_of::<libc::signalfd_siginfo>()
        ))?;
        if len != size_of::<libc::signalfd_siginfo>() {
            return Ok(None);
        }
        let fdsi = unsafe { fdsi.assume_init() };
        Ok(Sig::from_sig(fdsi.ssi_signo as i32))
    }
}
