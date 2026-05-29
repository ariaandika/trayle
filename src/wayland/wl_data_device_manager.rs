use crate::wayland::prelude::*;

// ===== Op =====

pub enum RequestOp {
    CreateDataSource,
    GetDataDevice,
    Release,
}

impl FromOp for RequestOp {
    fn from_op(op: u16) -> Result<Self, WlError> {
        match op {
            0 => Ok(Self::CreateDataSource),
            1 => Ok(Self::GetDataDevice),
            2 => Ok(Self::Release),
            _ => Err(WlError::UnknownOp),
        }
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
