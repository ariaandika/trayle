use crate::wayland::prelude::*;

#[derive(Debug, Interface)]
pub struct WlDataDevice {
    id: ObjectId,
}
