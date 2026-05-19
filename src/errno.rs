use std::task::Poll;

pub type Result<T, E = Errno> = std::result::Result<T, E>;

/// Converts syscall return code to `Result`.
pub fn cvt<I, T: SyscallReturns<I>>(int: I) -> Result<T> {
    T::cvt(int)
}

pub fn cvtnb<I, T: SyscallNonBlocking<I>>(int: I) -> Poll<Result<T>> {
    T::cvtnb(int)
}

/// Converts syscall return code to `Result`, and capture `EWOULDBLOCK` to `Poll`.
macro_rules! ready {
    ($e:expr) => {
        match crate::errno::cvtnb(unsafe { $e }) {
            Poll::Ready(Ok(ok)) => ok,
            Poll::Ready(Err(err)) => return Poll::Ready(Err(err.into())),
            Poll::Pending => return Poll::Pending,
        }
    };
}

pub(crate) use ready;

// ===== cvt =====

pub trait SyscallReturns<I>: Sized {
    fn cvt(int: I) -> Result<Self>;
}

impl SyscallReturns<i32> for () {
    fn cvt(int: i32) -> Result<Self> {
        if int != -1 { Ok(()) } else { Err(Errno {}) }
    }
}

impl SyscallReturns<i32> for usize {
    fn cvt(int: i32) -> Result<Self> {
        match usize::try_from(int) {
            Ok(ok) => Ok(ok),
            Err(_) => Err(Errno {}),
        }
    }
}

impl SyscallReturns<i32> for std::os::fd::OwnedFd {
    fn cvt(int: i32) -> Result<Self> {
        if int >= 0 {
            Ok(unsafe { std::os::fd::FromRawFd::from_raw_fd(int) })
        } else {
            Err(Errno {})
        }
    }
}

impl SyscallReturns<isize> for usize {
    fn cvt(int: isize) -> Result<Self> {
        match usize::try_from(int) {
            Ok(ok) => Ok(ok),
            Err(_) => Err(Errno {}),
        }
    }
}

// ===== cvtnb =====

pub trait SyscallNonBlocking<I>: Sized {
    fn cvtnb(int: I) -> Poll<Result<Self>>;
}

impl SyscallNonBlocking<isize> for usize {
    fn cvtnb(int: isize) -> Poll<Result<Self>> {
        match usize::try_from(int) {
            Ok(ok) => Poll::Ready(Ok(ok)),
            Err(_) => {
                if errno() == libc::EWOULDBLOCK {
                    Poll::Pending
                } else {
                    Poll::Ready(Err(Errno {}))
                }
            }
        }
    }
}

impl SyscallNonBlocking<i32> for std::os::fd::OwnedFd {
    fn cvtnb(int: i32) -> Poll<Result<Self>> {
        if int >= 0 {
            Poll::Ready(Ok(unsafe { std::os::fd::FromRawFd::from_raw_fd(int) }))
        } else if errno() == libc::EWOULDBLOCK {
            Poll::Pending
        } else {
            Poll::Ready(Err(Errno {}))
        }
    }
}

// ===== utils =====

fn errno() -> i32 {
    unsafe { *libc::__errno_location() }
}

/// Returns `Ok` if `errno` is `EINTR`.
pub fn not_interupt<T>(ok: T) -> Result<T> {
    if errno() == libc::EINTR {
        Ok(ok)
    } else {
        Err(Errno {})
    }
}

pub struct Errno {}
