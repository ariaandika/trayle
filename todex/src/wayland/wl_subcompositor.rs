use crate::wayland::prelude::*;
use crate::wayland::wl_subsurface::WlSubsurface;

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
#[request(WlSubcompositor)]
pub struct Destroy;

#[derive(Message, Debug)]
#[request(WlSubcompositor)]
pub struct GetSubsurface {
    pub wl_subsurface: NewId<WlSubsurface>,
    /// <wl_surface>
    pub surface: ObjectId,
    /// <wl_surface>
    pub parent: ObjectId,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    BadSurface,
    BadParent,
}
