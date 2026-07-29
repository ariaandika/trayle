//! Error code handling and representation.
use std::ffi::CStr;
use std::fmt;
use std::num::NonZeroU8;
use std::task::Poll;

// ===== OsError =====

/// A extension trait that represent an os error.
///
/// Use [`simple_os_error`] for simple [`ErrCode`] wrapper error.
pub trait OsError: Sized {
    /// A context string.
    ///
    /// Context is a short sentence that will be logged with prefix such "failed to" or "cannot".
    const CONTEXT: &str;

    /// The struct name.
    ///
    /// Currently, this is only used in `Debug` implementation.
    const NAME: &str;

    /// Create self from error code.
    fn from_code(code: ErrCode) -> Self;

    /// Returns the contained error code.
    fn code(&self) -> ErrCode;

    /// Create self with error code from `errno`.
    #[inline]
    fn errno() -> Self {
        Self::from_code(ErrCode::errno())
    }
}

/// Implement [`OsError`] for single field tuple struct [`ErrCode`] wrapper.
///
/// This also implement [`Debug`], [`Display`] and [`Error`].
///
/// [`Debug`]: fmt::Debug
/// [`Display`]: fmt::Display
/// [`Error`]: std::error::Error
macro_rules! simple_os_error {
    ($me:ident, $c:expr) => { const _: () = {
        use crate::sys::error::{OsError, ErrCode};
        use std::fmt;
        impl OsError for $me {
            const CONTEXT: &str = $c;
            const NAME: &str = stringify!($me);
            #[inline]
            fn from_code(code: ErrCode) -> Self { Self(code) }
            #[inline]
            fn code(&self) -> ErrCode { self.0 }
        }

        impl fmt::Display for $me {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "failed to {}: {}", Self::CONTEXT, self.code())
            }
        }

        impl fmt::Debug for $me {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple(Self::NAME).field(&self.code()).finish()
            }
        }

        impl std::error::Error for $me { }

        impl From<ErrCode> for $me {
            #[inline]
            fn from(v: ErrCode) -> Self {
                Self(v)
            }
        }
    };};
}
pub(crate) use simple_os_error;

// ===== ErrCode =====

/// Raw Error Code.
#[derive(Debug, Clone, Copy)]
pub struct ErrCode(NonZeroU8);

impl ErrCode {
    /// Create [`ErrCode`] with value from `errno`.
    #[inline]
    pub fn errno() -> Self {
        Self(NonZeroU8::new(Self::raw_errno() as _).unwrap_or(NonZeroU8::MAX))
    }

    /// Returns raw error code from `__errno_location`.
    #[inline]
    pub fn raw_errno() -> i32 {
        unsafe { *libc::__errno_location() }
    }

    /// Returns the contained raw error code.
    #[inline]
    pub fn code(self) -> i32 {
        self.0.get() as i32
    }

    /// Returns `true` if error code is `EWOULDBLOCK` or `EAGAIN`.
    #[inline]
    pub fn would_block(self) -> bool {
        // `EAGAIN` is the same as `EWOULDBLOCK`
        matches!(self.code(), libc::EWOULDBLOCK)
    }
}

impl ErrCode {
    /// Returns [`Pending`] if [`would_block`] returns `true`.
    ///
    /// Otherwise, returns [`Ready(Err(E))`] with `errno`.
    ///
    /// This can be used for converting non-blocking call error to [`Poll`].
    ///
    /// Note that this will eagerly retrieve `errno`, make sure to call this after `errno` is set.
    ///
    /// [`would_block`]: ErrCode::would_block
    /// [`Pending`]: Poll::Pending
    /// [`Ready(Err(E))`]: Poll::Ready
    #[inline]
    pub fn would_block_or<T, E: From<ErrCode>>() -> Poll<Result<T, E>> {
        let code = Self::errno();
        if code.would_block() {
            Poll::Pending
        } else {
            Poll::Ready(Err(E::from(code)))
        }
    }

    /// Returns [`Pending`] if [`would_block`] returns `true`.
    ///
    /// Otherwise, returns [`Ready(Err(E))`] with given error.
    ///
    /// This can be used for converting non-blocking call error to [`Poll`].
    ///
    /// Note that this will eagerly retrieve `errno`, make sure to call this after `errno` is set.
    ///
    /// [`would_block`]: ErrCode::would_block
    /// [`Pending`]: Poll::Pending
    /// [`Ready(Err(E))`]: Poll::Ready
    #[inline]
    pub fn would_block_or_else<T, E, F: FnOnce(Self) -> E>(f: F) -> Poll<Result<T, E>> {
        let code = Self::errno();
        if code.would_block() {
            Poll::Pending
        } else {
            Poll::Ready(Err(f(code)))
        }
    }

    /// Returns `Ok` if the fd is not `-1`.
    ///
    /// Otherwise, returns `Err` with error code from `errno`.
    #[inline]
    pub fn from_raw_fd<E: From<ErrCode>, F: std::os::fd::FromRawFd>(fd: i32) -> Result<F, E> {
        if fd != -1 {
            Ok(unsafe { F::from_raw_fd(fd) })
        } else {
            Err(E::from(Self::errno()))
        }
    }
}

impl fmt::Display for ErrCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        let mut buf = [0u8; 128];
        let msg = unsafe {
            let res = libc::strerror_r(code, buf.as_mut_ptr().cast(), buf.len());
            if res >= 0 {
                CStr::from_bytes_with_nul_unchecked(&buf[..]).to_string_lossy()
            } else {
                "Unknown error code".into()
            }
        };
        write!(f, "{msg} (os error {})", code)
    }
}

// ===== ResCode =====

/// Result code.
///
/// This is wrapper of `i32` that gives additional method based on common pattern used in syscall or
/// ffi, where zero or positive value represent successful operation, while negative value represent
/// failure operation.
///
/// This uses `#[repr(transparent)]` and has representation of [`i32`] so it can be used in FFI as a
/// return type.
#[repr(transparent)]
pub struct ResCode(i32);

impl ResCode {
    /// Returns `Ok(())` if the result code is `0`.
    ///
    /// Otherwise, returns `Err` with value from `errno` location.
    pub fn ok<E: From<ErrCode>>(self) -> Result<(), E> {
        if self.0 == 0 {
            Ok(())
        } else {
            Err(ErrCode::errno().into())
        }
    }

    /// Returns `Ok(result)` if the result code is zero or positive.
    ///
    /// Otherwise, returns `Err` with value from `errno` location.
    pub fn uint<T: From<u32>, E: From<ErrCode>>(self) -> Result<T, E> {
        u32::try_from(self.0).map_or_else(|_| Err(ErrCode::errno().into()), |ok| Ok(ok.into()))
    }
}
