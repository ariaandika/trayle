use std::os::fd::AsRawFd;

use crate::sys::errno::Errno;
use crate::sys::memfd::Memfd;
use crate::wayland::{Message, wl_keyboard};

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

    pub const fn from_u32(flags: u32) -> Self {
        Self(flags)
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

static STATIC_XKB: &str = include_str!("../static-xkb");
const SIZE: u32 = STATIC_XKB.len() as u32;

pub struct Seat {
    capability: Capability,
    memfd: Memfd,
}

impl Seat {
    pub fn new() -> Result<Self, SeatError> {
        let memfd = Memfd::new().map_err(|_| SeatError::MemfdCreate)?;
        memfd
            .write_all(STATIC_XKB.as_bytes())
            .map_err(|_| SeatError::MemfdWrite)?;

        Ok(Self {
            capability: Capability::new().add_pointer().add_keyboard(),
            memfd,
        })
    }

    pub fn capability(&self) -> Capability {
        self.capability
    }

    pub const fn keymap_size(&self) -> u32 {
        SIZE
    }

    pub fn to_keymap_event(
        &self,
        wl_keyboard: &wl_keyboard::WlKeyboard,
    ) -> Message<wl_keyboard::Keymap> {
        wl_keyboard.keymap(
            wl_keyboard::KeymapFormat::XkbV1,
            self.memfd.as_raw_fd(),
            self.keymap_size(),
        )
    }
}

// ===== Error =====

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
        Errno.fmt(f)
    }
}
