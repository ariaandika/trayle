use crate::wayland::prelude::*;

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
    /// <wl_seat>
    pub seat: ObjectId,
    pub serial: u32,
}

#[derive(Message, Debug)]
#[request(XdgPopup)]
pub struct Reposition {
    /// <xdg_positioner>
    pub positioner: ObjectId,
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
