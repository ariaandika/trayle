fn errno() -> i32 {
    unsafe { *libc::__errno_location() }
}

// ===== Errno =====

#[derive(Default, Clone, Copy)]
pub struct Errno;

impl Errno {
    pub fn get() -> i32 {
        errno()
    }
}

impl std::error::Error for Errno { }

impl std::fmt::Display for Errno {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::io::Error::last_os_error().fmt(f)
    }
}

impl std::fmt::Debug for Errno {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::io::Error::last_os_error().fmt(f)
    }
}

// ===== macros =====

macro_rules! simple_errno {
    ($v:vis $name:ident, $m:literal) => {
        #[derive(Default)]
        $v struct $name;

        impl std::error::Error for $name {}

        impl From<crate::sys::errno::Errno> for $name {
            fn from(_: crate::sys::errno::Errno) -> Self {
                Self
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, $m, crate::sys::errno::Errno)
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{self}")
            }
        }
    };
    ($($v:vis $name:ident, $m:literal;)*) => {
        $(crate::sys::errno::simple_errno!($v $name, $m);)*
    };
    () => {}
}

pub(crate) use simple_errno;
