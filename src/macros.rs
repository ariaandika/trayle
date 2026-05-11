
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
                Err(std::io::Error::last_os_error())
            }
        }
    };
    ($f:ident($($tt:tt)*)) => {
        {
            #[allow(unused_unsafe)]
            let result = unsafe { libc::$f($($tt)*) };
            match usize::try_from(result) {
                Ok(ok) => Ok(ok),
                Err(_) => Err(io::Error::last_os_error()),
            }
        }
    };
}

pub(crate) use {syscall};

