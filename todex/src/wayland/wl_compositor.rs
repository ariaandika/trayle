use crate::wayland::prelude::*;
use crate::wayland::wl_region::WlRegion;
use crate::wayland::wl_surface::WlSurface;

#[derive(Interface, Debug)]
#[interface(global = 7)]
pub struct WlCompositor {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    CreateSurface,
    CreateRegion,
    Release,
}

#[derive(Message, Debug)]
#[message(request = WlCompositor)]
pub struct CreateSurface {
    pub surface: NewId<WlSurface>,
}

#[derive(Message, Debug)]
#[message(request = WlCompositor)]
pub struct CreateRegion {
    pub region: NewId<WlRegion>,
}

#[derive(Message, Debug)]
#[message(request = WlCompositor, destructor)]
pub struct Release;
