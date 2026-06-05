use crate::wayland::prelude::*;

#[derive(Debug, Interface)]
pub struct WlDataSource {
    id: ObjectId,
}
