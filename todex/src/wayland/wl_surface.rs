use crate::wayland::prelude::*;

#[derive(Debug, Interface)]
pub struct WlSurface {
    id: ObjectId,
}
