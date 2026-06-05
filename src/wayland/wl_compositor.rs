use crate::wayland::prelude::*;
use crate::wayland::wl_surface::WlSurface;

#[derive(Interface, Debug)]
pub struct WlCompositor {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    CreateSurface,
}

#[derive(Message, Debug)]
#[request(WlCompositor)]
pub struct CreateSurface {
    pub surface: NewId<WlSurface>,
}
