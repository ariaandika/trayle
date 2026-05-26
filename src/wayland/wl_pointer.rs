use crate::wayland::prelude::*;

#[derive(Debug)]
pub struct Pointer {
    id: Id,
}

impl Object for Pointer {
    const INTERFACE_ID: InterfaceId = InterfaceId::WlPointer;

    fn id(&self) -> Id {
        self.id
    }
}

impl Pointer {
    /// Can only be created by `wl_seat::get_pointer`.
    pub(super) fn new(id: Id) -> Self {
        Self { id }
    }
}
