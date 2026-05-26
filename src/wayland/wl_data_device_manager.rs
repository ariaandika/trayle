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

    fn decode<'a>(reader: &mut Reader<'a>) -> Result<Self::Output<'a>, WlError> {
        Ok(Self { id: reader.read()? })
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

    fn decode<'a>(reader: &mut Reader<'a>) -> Result<Self::Output<'a>, WlError> {
        Ok(Self {
            id: reader.read()?,
            seat: reader.read()?,
        })
    }
}

// ===== Release =====

#[derive(Debug)]
pub struct Release;

impl Decode for Release {
    type Output<'a> = Self;

    fn decode<'a>(_: &mut Reader<'a>) -> Result<Self::Output<'a>, WlError> {
        Ok(Self)
    }
}

