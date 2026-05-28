use crate::wayland::prelude::*;

#[derive(Debug)]
pub struct WlSurface {
    id: Id,
}

impl Object for WlSurface {
    const INTERFACE_ID: InterfaceId = InterfaceId::WlSurface;

    fn id(&self) -> Id {
        self.id
    }
}

impl WlSurface {
    pub fn new(id: Id) -> Self {
        Self { id }
    }
}

