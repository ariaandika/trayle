use std::os::fd::{AsFd, AsRawFd};
use todex::sys::memfd::Memfd;
use todex::sys::xkb::{self, KeymapFormat};
use todex::wayland::interface::wl_seat::Capability;

use crate::error::FatalError;

// ===== Seat =====

pub struct Seat {
    name: Box<str>,
    capability: Capability,
    data_device: Option<u64>,
    keymap_memfd: Memfd,
    keymap_size: u32,
}

impl Seat {
    pub fn new() -> Result<Self, FatalError> {
        let xkb = xkb::Xkb::new(<_>::default())?;
        let keymap = xkb::Keymap::new_from_names(&xkb, None, KeymapFormat::TextV1, <_>::default())?;
        let string = keymap.to_string(KeymapFormat::TextV1, <_>::default())?;
        let bytes = string.to_bytes_with_nul();

        let memfd = Memfd::new()?;
        memfd.write_all(bytes)?;

        Ok(Self {
            name: String::from("seat0").into_boxed_str(),
            capability: Capability::POINTER | Capability::KEYBOARD,
            data_device: None,
            keymap_memfd: memfd,
            keymap_size: bytes.len() as u32,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn capability(&self) -> Capability {
        self.capability
    }

    pub fn keymap_memfd(&self) -> i32 {
        self.keymap_memfd.as_fd().as_raw_fd()
    }

    pub const fn keymap_size(&self) -> u32 {
        self.keymap_size
    }

    /// Set client id that holds the data device.
    pub fn set_data_device(&mut self, client_id: u64) {
        self.data_device = Some(client_id);
    }

    /// Clear client id that holds the data device.
    pub fn clear_data_device(&mut self) {
        self.data_device = None;
    }
}
