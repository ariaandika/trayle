use crate::wayland::prelude::*;

const GLOBAL_OP: u16 = 0;

pub struct WlRegistry {
    id: Id,
}

impl WlRegistry {
    /// Can only be created by `GetRegistry`.
    pub(super) fn new(id: Id) -> Self {
        Self { id }
    }

    /// Send `wl_registry::global` event.
    pub fn global(&self, name: u32, interface: &str, version: u32, buffer: &mut Buffer) {
        let iface_len = interface.len() as u16;
        let len = const { 8 + 4 + 4 + 4 } + roundup4!(iface_len + 1);
        unsafe {
            buffer
                .message(self.id, GLOBAL_OP, len)
                .put(name)
                .put(interface)
                .put(version)
        };
    }
}
