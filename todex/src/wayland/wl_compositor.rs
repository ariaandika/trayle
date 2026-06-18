use crate::wayland::prelude::*;
use crate::wayland::wl_region::WlRegion;
use crate::wayland::wl_surface::WlSurface;

#[derive(Interface, Debug)]
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
#[request(WlCompositor)]
pub struct CreateSurface {
    pub surface: NewId<WlSurface>,
}

#[derive(Message, Debug)]
#[request(WlCompositor)]
pub struct CreateRegion {
    pub region: NewId<WlRegion>,
}

#[derive(Message, Debug)]
#[request(WlCompositor)]
#[destructor]
pub struct Release;
