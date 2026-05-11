

/// cope and seeth
macro_rules! try_block {
    ($($tt:tt)*) => {
        (||{$($tt)*})()
    };
}

macro_rules! syscall {
    ($f:ident $u:tt, $($tt:tt)*) => {
        {
            #[allow(unused_unsafe)]
            let result = unsafe { libc::$f($($tt)*) };
            match $u::try_from(result) {
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
    ($f:ident($($tt:tt)*)) => {
        crate::macros::syscall!($f,$($tt)*)
    };
}

pub(crate) use {try_block, syscall};

