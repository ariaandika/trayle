use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

const POINTER: u32 = 1;
const KEYBOARD: u32 = 1 << 1;
// const TOUCH: u32 = 1 << 2;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Capability(u32);

impl Capability {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn add_pointer(self) -> Self {
        Self(self.0 | POINTER)
    }

    pub const fn add_keyboard(self) -> Self {
        Self(self.0 | KEYBOARD)
    }

    pub const fn to_u32(self) -> u32 {
        self.0
    }
}

// ===== Seat =====

static STATIC_XKB: &str = include_str!("./static-xkb");
const SIZE: u32 = STATIC_XKB.len() as u32;

pub struct Seat {
    capability: Capability,
    keymap_memfd: OwnedFd,
}

impl Seat {
    pub fn new() -> Result<Self, SeatError> {
        let fd = unsafe { libc::memfd_create(c"wayland-keymap".as_ptr(), 0) };
        if fd == -1 {
            return Err(SeatError::MemfdCreate);
        }
        let fd = unsafe { OwnedFd::from_raw_fd(fd) };

        let mut writen = 0;
        while let Some(chunk) = STATIC_XKB.as_bytes().get(writen..)
            && !chunk.is_empty()
        {
            let result = unsafe { libc::write(fd.as_raw_fd(), chunk.as_ptr().cast(), chunk.len()) };
            let Ok(write) = usize::try_from(result) else {
                return Err(SeatError::MemfdWrite);
            };
            writen += write;
        }

        Ok(Self {
            capability: Capability::new().add_pointer().add_keyboard(),
            keymap_memfd: fd,
        })
    }

    pub fn capability(&self) -> Capability {
        self.capability
    }

    pub fn keymap_memfd(&self) -> i32 {
        self.keymap_memfd.as_raw_fd()
    }

    pub const fn keymap_size(&self) -> u32 {
        SIZE
    }
}

#[derive(Debug)]
pub enum SeatError {
    MemfdCreate,
    MemfdWrite,
}

impl std::error::Error for SeatError {}

impl std::fmt::Display for SeatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MemfdCreate => write!(f, "failed to create memfd: ")?,
            Self::MemfdWrite => write!(f, "failed to write to memfd: ")?,
        }
        crate::sys::errno::Errno.fmt(f)
    }
}
