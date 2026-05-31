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
    fn encode(self, mut encoder: Encoder) {
        encoder.push_fd(self.fd);
        let mut writer = unsafe { encoder.encode(self.id(), EventOp::Keymap, 16) };
        writer.write(self.format as u32).write(self.size);
    }
}
