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


