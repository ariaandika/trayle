use crate::wayland::prelude::*;
use crate::wayland::wl_surface::WlSurface;

#[derive(Interface, Debug)]
pub struct WlSubsurface {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Destroy,
    SetPosition,
    PlaceAbove,
    PlaceBelow,
    SetSync,
    SetDesync,
}

#[derive(Message, Debug)]
#[message(request = WlSubsurface, destructor)]
pub struct Destroy;

#[derive(Message, Debug)]
#[message(request = WlSubsurface)]
pub struct SetPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Message, Debug)]
#[message(request = WlSubsurface)]
pub struct PlaceAbove {
    pub sibling: Object<WlSurface>,
}

#[derive(Message, Debug)]
#[message(request = WlSubsurface)]
pub struct PlaceBelow {
    pub sibling: Object<WlSurface>,
}

#[derive(Message, Debug)]
#[message(request = WlSubsurface)]
pub struct SetSync;

#[derive(Message, Debug)]
#[message(request = WlSubsurface)]
pub struct SetDesync;

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    BadSurface,
}
