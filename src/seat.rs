use std::os::fd::AsRawFd;

use todex::sys::errno::Errno;
use todex::sys::memfd::Memfd;
use todex::wayland::{Encodable, wl_keyboard};
use todex::wayland::wl_seat::Capability;

// ===== Seat =====

static STATIC_XKB: &str = include_str!("./static-xkb");

pub struct Seat {
    name: Box<str>,
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
            name: String::from("seat0").into_boxed_str(),
            capability: Capability::POINTER | Capability::KEYBOARD,
            data_device: None,
            memfd,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn capability(&self) -> Capability {
        self.capability
    }

    pub const fn keymap_size(&self) -> u32 {
        STATIC_XKB.len() as u32
    }

    /// Set client id that holds the data device.
    pub fn set_data_device(&mut self, client_id: u64) {
        self.data_device = Some(client_id);
    }

    /// Clear client id that holds the data device.
    pub fn clear_data_device(&mut self) {
        self.data_device = None;
    }

    pub fn to_keymap_event(
        &self,
        format: wl_keyboard::KeymapFormat,
        wl_keyboard: &wl_keyboard::WlKeyboard,
    ) -> Encodable<wl_keyboard::Keymap> {
        wl_keyboard.keymap(format, self.memfd.as_raw_fd(), self.keymap_size())
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
