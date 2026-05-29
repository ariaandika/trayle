use crate::wayland::prelude::*;

#[derive(Debug)]
pub struct WlSurface {
    id: Id,
}

impl FromId for WlSurface {
    fn from_id(id: Id) -> Self {
        Self { id }
    }
}

impl Object for WlSurface {
    const INTERFACE_ID: InterfaceId = InterfaceId::WlSurface;

    fn id(&self) -> Id {
        self.id
    }
}
