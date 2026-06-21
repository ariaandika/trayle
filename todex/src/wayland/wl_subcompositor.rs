use crate::wayland::prelude::*;
use crate::wayland::wl_subsurface::WlSubsurface;
use crate::wayland::wl_surface::WlSurface;

#[derive(Interface, Debug)]
pub struct WlSubcompositor {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Destroy,
    GetSubsurface,
}

#[derive(Message, Debug)]
#[message(request = WlSubcompositor, destructor)]
pub struct Destroy;

#[derive(Message, Debug)]
#[message(request = WlSubcompositor)]
pub struct GetSubsurface {
    pub wl_subsurface: NewId<WlSubsurface>,
    pub surface: Object<WlSurface>,
    pub parent: Object<WlSurface>,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    BadSurface,
    BadParent,
}
