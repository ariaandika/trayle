use crate::wayland::prelude::*;

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
#[request(WlSubsurface)]
pub struct Destroy;

#[derive(Message, Debug)]
#[request(WlSubsurface)]
pub struct SetPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Message, Debug)]
#[request(WlSubsurface)]
pub struct PlaceAbove {
    /// <wl_surface>
    pub sibling: ObjectId,
}

#[derive(Message, Debug)]
#[request(WlSubsurface)]
pub struct PlaceBelow {
    /// <wl_surface>
    pub sibling: ObjectId,
}

#[derive(Message, Debug)]
#[request(WlSubsurface)]
pub struct SetSync;

#[derive(Message, Debug)]
#[request(WlSubsurface)]
pub struct SetDesync;

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    BadSurface,
}
