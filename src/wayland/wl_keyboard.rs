use crate::seat::Seat;
use crate::wayland::prelude::*;

#[derive(Debug)]
pub struct Keyboard {
    id: Id,
}

impl Object for Keyboard {
    const INTERFACE_ID: InterfaceId = InterfaceId::WlKeyboard;

    #[inline]
    fn id(&self) -> Id {
        self.id
    }
}

impl Keyboard {
    #[inline]
    pub fn new(id: Id) -> Self {
        Self { id }
    }

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

#[derive(Debug, Clone, Copy)]
pub enum EventOp {
    Keymap,
}

impl ToOp for EventOp {
    fn to_op(&self) -> u16 {
        *self as u16
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
