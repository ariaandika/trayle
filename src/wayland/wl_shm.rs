use crate::wayland::prelude::*;

// ===== Op =====

pub enum RequestOp {
    CreatePool,
    Release,
}

impl FromOp for RequestOp {
    fn from_op(op: u16) -> Result<Self, WlError> {
        match op {
            0 => Ok(Self::CreatePool),
            1 => Ok(Self::Release),
            _ => Err(WlError::UnknownOp),
        }
    }
}

// ===== CreatePool =====

#[derive(Debug)]
#[allow(dead_code)]
pub struct CreatePool {
    /// <wl_shm_pool>
    pub id: Id,
    pub fd: i32,
    pub size: i32,
}

impl Decode for CreatePool {
    type Output<'a> = Self;

    fn decode(mut decoder: Decoder<'_>) -> Result<Self::Output<'_>, WlError> {
        let fd = decoder.pop_fd()?;
        let mut reader = decoder.body();
        Ok(Self {
            id: reader.read()?,
            fd,
            size: reader.read()?,
        })
    }
}
