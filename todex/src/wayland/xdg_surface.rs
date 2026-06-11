use crate::wayland::prelude::*;
use crate::wayland::xdg_popup::XdgPopup;
use crate::wayland::xdg_toplevel::XdgToplevel;

#[derive(Interface, Debug)]
pub struct XdgSurface {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Destroy,
    GetToplevel,
    GetPopup,
    SetWindowGeometry,
    AckConfigure,
}

#[derive(Message, Debug)]
#[request(XdgSurface)]
pub struct Destroy;

#[derive(Message, Debug)]
#[request(XdgSurface)]
pub struct GetToplevel {
    pub id: NewId<XdgToplevel>,
}

#[derive(Message, Debug)]
#[request(XdgSurface)]
pub struct GetPopup {
    pub id: NewId<XdgPopup>,
    /// <xdg_surface>
    pub parent: Option<ObjectId>,
    /// <xdg_positioner>
    pub positioner: ObjectId,
}

#[derive(Message, Debug)]
#[request(XdgSurface)]
pub struct SetWindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Message, Debug)]
#[request(XdgSurface)]
pub struct AckConfigure {
    pub serial: u32,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Configure,
}

#[derive(Message, Debug)]
#[event(XdgSurface)]
pub struct Configure {
    pub serial: u32,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    /// Surface was not fully constructed
    NotConstructed,
    /// Surface was already constructed
    AlreadyConstructed,
    /// Attaching a buffer to an unconfigured surface
    UnconfiguredBuffer,
    /// Invalid serial number when acking a configure event
    InvalidSerial,
    /// Width or height was zero or negative
    InvalidSize,
    /// Surface was destroyed before its role object
    DefunctRoleObject,
}
