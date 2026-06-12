use crate::wayland::prelude::*;
use crate::wayland::wl_seat::WlSeat;
use crate::wayland::xdg_positioner::XdgPositioner;

#[derive(Interface, Debug)]
pub struct XdgPopup {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Destroy,
    Grab,
    Reposition,
}

#[derive(Message, Debug)]
#[request(XdgPopup)]
pub struct Grab {
    pub seat: Object<WlSeat>,
    pub serial: u32,
}

#[derive(Message, Debug)]
#[request(XdgPopup)]
pub struct Reposition {
    pub positioner: Object<XdgPositioner>,
    pub token: u32,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Configure,
    PopupDone,
    Repositioned,
}

#[derive(Message, Debug)]
#[event(XdgPopup)]
pub struct Configure {
    pub serial: u32,
}

#[derive(Message, Debug)]
#[event(XdgPopup)]
pub struct PopupDone;

#[derive(Message, Debug)]
#[event(XdgPopup)]
pub struct Repositioned {
    pub token: u32,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    /// tried to grab after being mapped
    InvalidGrab,
}
