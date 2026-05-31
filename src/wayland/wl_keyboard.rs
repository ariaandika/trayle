use crate::wayland::prelude::*;

simple_object! {
    pub struct WlKeyboard;
}

impl WlKeyboard {
    /// Send `wl_keyboard::keymap` event.
    pub fn keymap(&self, format: KeymapFormat, fd: i32, size: u32) -> Message<Keymap> {
        Message::new(self, Keymap { format, fd, size })
    }
}

// ===== Op =====

opcode! {
    pub enum EventOp {
        Keymap,
    }
}

// ===== Keymap =====

#[derive(Debug, Clone, Copy)]
pub enum KeymapFormat {
    XkbV1
}

pub struct Keymap {
    format: KeymapFormat,
    fd: i32,
    size: u32,
}

impl Encode for Message<Keymap> {
    const OPCODE: u16 = EventOp::Keymap as u16;

    #[inline]
    fn object_id(&self) -> Id {
        self.id()
    }

    #[inline]
    fn encode(self, mut encoder: Encoder) {
        encoder.push_fd(self.fd);
        encode_me!(encoder, self, format, size);
    }
}

impl WaylandEnum for KeymapFormat {
    #[inline]
    fn to_u32(self) -> u32 {
        self as u32
    }
}
