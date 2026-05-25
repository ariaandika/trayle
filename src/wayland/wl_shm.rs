use crate::wayland::prelude::*;

// ===== Op =====

pub struct Op;

impl FromOpCode for Op {
    type RequestOp = RequestOp;

    fn from_request_op(op: u16) -> Result<Self::RequestOp, WlError> {
        use RequestOp as Op;
        match op {
            0 => Ok(Op::CreatePool(Decoder::new())),
            1 => Ok(Op::Release(Decoder::new())),
            _ => Err(WlError::UnknownOp),
        }
    }
}

pub enum RequestOp {
    CreatePool(Decoder<CreatePool>),
    Release(Decoder<Release>),
}

// ===== CreatePool =====

pub struct CreatePool {
    /// <wl_shm_pool>
    pub id: Id,
    pub fd: i32,
    pub size: i32,
}

impl Decode for CreatePool {
    type Output<'a> = Self;

    fn decode<'a>(reader: &mut Reader<'a>) -> Result<Self::Output<'a>, WlError> {
        Ok(Self {
            id: reader.read()?,
            fd: reader.read_fd()?,
            size: reader.read_int()?,
        })
    }
}

// ===== Release =====

pub struct Release;

impl Decode for Release {
    type Output<'a> = Self;

    fn decode<'a>(_: &mut Reader<'a>) -> Result<Self::Output<'a>, WlError> {
        Ok(Self)
    }
}

