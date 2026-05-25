use crate::wayland::prelude::*;


// ===== DataDeviceManager =====

pub struct Op;

impl FromOpCode for Op {
    type RequestOp = RequestOp;

    fn from_request_op(op: u16) -> Result<Self::RequestOp, WlError> {
        match op {
            0 => Ok(RequestOp::CreateDataSource),
            1 => Ok(RequestOp::GetDataDevice(Decoder::new())),
            2 => Ok(RequestOp::Release),
            _ => Err(WlError::UnknownOp),
        }
    }
}

pub enum RequestOp {
    CreateDataSource,
    GetDataDevice(Decoder<GetDataDevice>),
    Release,
}

// ===== GetDataDevice =====

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
