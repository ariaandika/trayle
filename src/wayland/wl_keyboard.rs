use crate::seat::Seat;
use crate::wayland::prelude::*;

simple_object! {
    pub struct WlKeyboard::Keyboard;
}

impl Keyboard {
    pub fn keymap_xkb_v1(&self, seat: &Seat) -> Message<Keymap> {
        Message::new(
            self,
            Keymap {
                format: KeymapFormat::XkbV1,
                fd: seat.keymap_memfd(),
                size: seat.keymap_size(),
            },
        )
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
    fn encode(self, mut encoder: Encoder) {
        encoder.push_fd(self.fd);
        let mut writer = unsafe { encoder.encode(self.id(), EventOp::Keymap, 16) };
        writer.write(self.format as u32).write(self.size);
    }
}
