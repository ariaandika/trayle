use crate::seat::Seat;
use crate::wayland::prelude::*;

#[derive(Debug)]
pub struct Keyboard {
    id: Id,
}

impl Object for Keyboard {
    const INTERFACE_ID: InterfaceId = InterfaceId::WlKeyboard;

    fn id(&self) -> Id {
        self.id
    }
}

impl Keyboard {
    /// Can only be created by `wl_seat::get_keyboard`.
    pub(super) fn new(id: Id) -> Self {
        Self { id }
    }
}

// ===== Keymap =====

pub fn keymap_xkb_v1(wl_keyboard_id: Id, seat: &Seat, buffer: &mut Buffer) {
    assert!(buffer.push_fd(seat.keymap_memfd()));
    unsafe {
        buffer
            .message(wl_keyboard_id, 0, 16)
            .put(KeymapFormat::XkbV1 as u32)
            .put(seat.keymap_size());
    }
}

pub enum KeymapFormat {
    XkbV1
}
