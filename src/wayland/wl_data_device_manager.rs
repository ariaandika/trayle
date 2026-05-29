use crate::wayland::prelude::*;

pub struct DataDeviceManager {
    id: Id,
}

impl FromId for DataDeviceManager {
    fn from_id(id: Id) -> Self {
        Self { id }
    }
}

impl Object for DataDeviceManager {
    const INTERFACE_ID: InterfaceId = InterfaceId::WlDataDeviceManager;

    #[inline]
    fn id(&self) -> Id {
        self.id
    }
}

// ===== Op =====

opcode! {
    pub enum RequestOp {
        CreateDataSource,
        GetDataDevice,
        Release,
    }
}

// ===== CreateDataSource =====

#[derive(Debug)]
#[allow(dead_code)]
pub struct CreateDataSource {
    // wl_data_source
    pub id: Id,
}

impl Decode for CreateDataSource {
    type Output<'a> = Self;

    fn decode(decoder: Decoder<'_>) -> Result<Self::Output<'_>, WlError> {
        Ok(Self { id: decoder.read()? })
    }
}

// ===== GetDataDevice =====

#[derive(Debug)]
pub struct GetDataDevice {
    // <wl_data_device>
    pub id: Id,
    // <wl_seat>
    pub seat: Id,
}

impl Decode for GetDataDevice {
    type Output<'a> = Self;

    fn decode(decoder: Decoder<'_>) -> Result<Self::Output<'_>, WlError> {
        let mut reader = decoder.body();
        Ok(Self {
            id: reader.read()?,
            seat: reader.read()?,
        })
    }
}
