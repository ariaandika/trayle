use std::os::fd::AsRawFd;

use crate::sys::errno::Errno;
use crate::sys::memfd::Memfd;
use crate::wayland::{Encodable, wl_keyboard};
use crate::wayland::wl_seat::Capability;

// ===== Seat =====

static STATIC_XKB: &str = include_str!("../static-xkb");
const SIZE: u32 = STATIC_XKB.len() as u32;

pub struct Seat {
    capability: Capability,
    data_device: Option<u64>,
    memfd: Memfd,
}

impl Seat {
    pub fn new() -> Result<Self, SeatError> {
        let memfd = Memfd::new().map_err(|_| SeatError::MemfdCreate)?;
        memfd
            .write_all(STATIC_XKB.as_bytes())
            .map_err(|_| SeatError::MemfdWrite)?;

        Ok(Self {
            capability: Capability::POINTER | Capability::KEYBOARD,
            data_device: None,
            memfd,
        })
    }

    pub fn capability(&self) -> Capability {
        self.capability
    }

    pub const fn keymap_size(&self) -> u32 {
        SIZE
    }

    /// Returns the client id that holds the data device.
    #[inline]
    pub fn data_device(&self) -> Option<u64> {
        self.data_device
    }

    /// Set client id that holds the data device.
    #[inline]
    pub fn set_data_device(&mut self, client_id: u64) {
        self.data_device = Some(client_id);
    }

    /// Clear client id that holds the data device.
    #[inline]
    pub fn clear_data_device(&mut self) {
        self.data_device = None;
    }

    pub fn to_keymap_event(
        &self,
        wl_keyboard: &wl_keyboard::WlKeyboard,
    ) -> Encodable<wl_keyboard::Keymap> {
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
