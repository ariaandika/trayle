use std::mem::MaybeUninit;
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd, RawFd};

use crate::errno::simple_errno;

// https://man7.org/linux/man-pages/man2/signalfd.2.html

// ===== Sig =====

pub enum Sig {
    Int,
    Term,
    Unknown,
}

impl Sig {
    const SIGNALS: [i32; 2] = [libc::SIGINT, libc::SIGTERM];

    fn from_signo(sig: i32) -> Sig {
        match sig {
            libc::SIGINT => Sig::Int,
            libc::SIGTERM => Sig::Term,
            _ => Sig::Unknown,
        }
    }
}

impl std::fmt::Display for Sig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sig::Int => "INT",
            Sig::Term => "TERM",
            Sig::Unknown => "unrecognized",
        }
        .fmt(f)
    }
}

// ===== Sigfd =====

pub struct Sigfd(OwnedFd);

impl AsRawFd for Sigfd {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

pub fn e(int: i32) -> Result<i32, CreateError> {
    if int != -1 { Ok(int) } else { Err(CreateError) }
}

impl Sigfd {
    pub fn new() -> Result<Self, CreateError> {
        unsafe {
            let mut mask = MaybeUninit::uninit();
            e(libc::sigemptyset(mask.as_mut_ptr()))?;
            for signal in Sig::SIGNALS {
                e(libc::sigaddset(mask.as_mut_ptr(), signal))?;
            }
            e(libc::sigprocmask(
                libc::SIG_BLOCK,
                mask.as_ptr(),
                std::ptr::null_mut(),
            ))?;
            let fd = e(libc::signalfd(-1, mask.as_ptr(), libc::SFD_NONBLOCK))?;
            Ok(Self(<_>::from_raw_fd(fd)))
        }
    }

    pub fn read(&self) -> Sig {
        const DATA_SIZE: usize = size_of::<libc::signalfd_siginfo>();

        let mut fdsi = MaybeUninit::<libc::signalfd_siginfo>::uninit();
        let read = unsafe {
            libc::read(
                self.0.as_raw_fd(),
                fdsi.as_mut_ptr().cast(),
                size_of::<libc::signalfd_siginfo>(),
            )
        };
        if read != DATA_SIZE as isize {
            return Sig::Unknown;
        }
        let fdsi = unsafe { fdsi.assume_init() };
        Sig::from_signo(fdsi.ssi_signo as i32)
    }
}

// ===== Error =====

simple_errno! {
    pub CreateError, "failed to create signalfd: {}";
}
