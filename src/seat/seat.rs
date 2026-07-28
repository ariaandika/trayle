use std::os::fd::{AsFd, AsRawFd};
use todex::sys::memfd::{CreateError, Memfd, WriteError};
use todex::wayland::interface::wl_seat::Capability;

use crate::seat::xkb::Xkb;

// ===== Seat =====

pub struct Seat {
    name: Box<str>,
    capability: Capability,
    data_device: Option<u64>,
    keymap_memfd: Memfd,
    xkb: Xkb,
}

impl Seat {
    pub fn new() -> Result<Self, SeatError> {
        let xkb = Xkb::new();

        let memfd = Memfd::new()?;
        memfd.write_all(xkb.keymap_str().to_bytes_with_nul())?;

        Ok(Self {
            name: String::from("seat0").into_boxed_str(),
            capability: Capability::POINTER | Capability::KEYBOARD,
            data_device: None,
            keymap_memfd: memfd,
            xkb,
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
        self.xkb.keymap_str().count_bytes() as u32 + 1
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

// ===== Error =====

#[derive(Debug)]
pub enum SeatError {
    MemfdCreate(CreateError),
    MemfdWrite(WriteError),
}

impl std::error::Error for SeatError {}

impl std::fmt::Display for SeatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MemfdCreate(err) => err.fmt(f),
            Self::MemfdWrite(err) => err.fmt(f),
        }
    }
}

impl From<CreateError> for SeatError {
    fn from(v: CreateError) -> Self {
        Self::MemfdCreate(v)
    }
}

impl From<WriteError> for SeatError {
    fn from(v: WriteError) -> Self {
        Self::MemfdWrite(v)
    }
}
