use crate::wayland::prelude::*;

// ===== capability =====

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
}

const EVENT_CAPABILITIES: u16 = 0;

impl Capability {
    pub fn encode(self, wl_seat: Id, write: &mut Buffer) {
        unsafe { write.message(wl_seat, EVENT_CAPABILITIES, 12).put(self.0) };
    }
}
