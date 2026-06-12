use crate::wayland::prelude::*;
use crate::wayland::wl_data_offer::WlDataOffer;
use crate::wayland::wl_data_source::WlDataSource;
use crate::wayland::wl_surface::WlSurface;

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
    pub source: Option<Object<WlDataSource>>,
    pub origin: Object<WlSurface>,
    pub icon: Option<Object<WlSurface>>,
    pub serial: u32,
}

#[derive(Message, Debug)]
#[request(WlDataDevice)]
pub struct SetSelection {
    pub source: Option<Object<WlDataSource>>,
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
    pub surface: Object<WlSurface>,
    pub x: Fixed,
    pub y: Fixed,
    pub id: Option<Object<WlDataOffer>>,
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
    pub id: Option<Object<WlDataOffer>>,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    /// given wl_surface has another role
    Role,
    /// source has already been used
    UsedSource,
}
