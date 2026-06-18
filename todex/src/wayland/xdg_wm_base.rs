use crate::wayland::prelude::*;
use crate::wayland::wl_surface::WlSurface;
use crate::wayland::xdg_positioner::XdgPositioner;
use crate::wayland::xdg_surface::XdgSurface;

#[derive(Interface, Debug)]
#[global(7)]
pub struct XdgWmBase {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Destroy,
    CreatePositioner,
    GetXdgSurface,
    Pong,
}

#[derive(Message, Debug)]
#[request(XdgWmBase)]
#[destructor]
pub struct Destroy;

#[derive(Message, Debug)]
#[request(XdgWmBase)]
pub struct CreatePositioner {
    pub positioner: NewId<XdgPositioner>,
}

#[derive(Message, Debug)]
#[request(XdgWmBase)]
pub struct GetXdgSurface {
    pub xdg_surface: NewId<XdgSurface>,
    pub wl_surface: Object<WlSurface>,
}

#[derive(Message, Debug)]
#[request(XdgWmBase)]
pub struct Pong {
    pub serial_unit: u32,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Ping,
}

#[derive(Message, Debug)]
#[event(XdgWmBase)]
pub struct Ping {
    pub serial_unit: u32,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    /// given wl_surface has another role
    Role,
    /// xdg_wm_base was destroyed before children
    DefunctSurfaces,
    /// the client tried to map or destroy a non-topmost popup
    NotTheTopmostPopup,
    /// the client specified an invalid popup parent surface
    InvalidPopupParent,
    /// the client provided an invalid surface state
    InvalidSurfaceState,
    /// the client provided an invalid positioner
    InvalidPositioner,
    /// the client didn’t respond to a ping event in time
    Unresponsive,
}
