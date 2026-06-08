use crate::wayland::prelude::*;
use crate::wayland::wl_data_device::WlDataDevice;
use crate::wayland::wl_data_source::WlDataSource;

#[derive(Interface, Debug)]
pub struct WlDataDeviceManager {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    CreateDataSource,
    GetDataDevice,
    Release,
}

#[derive(Message, Debug)]
#[request(WlDataDeviceManager)]
pub struct CreateDataSource {
    pub data_source: NewId<WlDataSource>,
}

#[derive(Message, Debug)]
#[request(WlDataDeviceManager)]
pub struct GetDataDevice {
    pub data_device: NewId<WlDataDevice>,
    /// <wl_seat>
    pub seat: ObjectId,
}

#[derive(Message, Debug)]
#[request(WlDataDeviceManager)]
pub struct Release;

/// TODO: bitfield
#[derive(WlEnum, Debug, Clone, Copy)]
pub enum DndAction {
    /// no action
    None,
    /// copy action
    Copy,
    /// move action
    Move,
    /// ask action
    Ask,
}
