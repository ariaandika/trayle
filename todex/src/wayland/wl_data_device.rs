use crate::wayland::prelude::*;
use crate::wayland::wl_data_offer::WlDataOffer;

#[derive(Debug, Interface)]
pub struct WlDataDevice {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    StartDrag,
    SetSelection,
    Release,
}

#[derive(Message, Debug)]
#[request(WlDataDevice)]
pub struct StartDrag {
    /// <wl_data_source>
    pub source: Option<ObjectId>,
    /// <wl_surface>
    pub origin: ObjectId,
    /// <wl_surface>
    pub icon: Option<ObjectId>,
    pub serial: u32,
}

#[derive(Message, Debug)]
#[request(WlDataDevice)]
pub struct SetSelection {
    /// <wl_data_source>
    pub source: Option<ObjectId>,
    pub serial: u32,
}

#[derive(Message, Debug)]
#[request(WlDataDevice)]
pub struct Release;

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    DataOffer,
    Enter,
    Leave,
    Motion,
    Drop,
    Selection,
}

#[derive(Message, Debug)]
#[event(WlDataDevice)]
pub struct DataOffer {
    pub wl_data_offer: NewId<WlDataOffer>,
}

#[derive(Message, Debug)]
#[event(WlDataDevice)]
pub struct Enter {
    pub serial: u32,
    /// <wl_surface>
    pub surface: ObjectId,
    pub x: Fixed,
    pub y: Fixed,
    /// <wl_data_offer>
    pub id: Option<ObjectId>,
}

#[derive(Message, Debug)]
#[event(WlDataDevice)]
pub struct Leave;

#[derive(Message, Debug)]
#[event(WlDataDevice)]
pub struct Motion {
    pub time: u32,
    pub x: Fixed,
    pub y: Fixed,
}

#[derive(Message, Debug)]
#[event(WlDataDevice)]
pub struct Drop;

#[derive(Message, Debug)]
#[event(WlDataDevice)]
pub struct Selection {
    /// <wl_data_offer>
    pub id: Option<ObjectId>,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    /// given wl_surface has another role
    Role,
    /// source has already been used
    UsedSource,
}
