use crate::wayland::prelude::*;
use crate::wayland::wl_surface::WlSurface;

pub enum RequestOp {
    CreateSurface,
}

impl FromOp for RequestOp {
    fn from_op(op: u16) -> Result<Self, WlError> {
        match op {
            0 => Ok(Self::CreateSurface),
            _ => Err(WlError::UnknownOp),
        }
    }
}

// ===== CreateSurace =====

#[derive(Debug)]
pub struct CreateSurface {
    /// `<wl_surface>`
    pub id: Id,
}

impl CreateSurface {
    pub fn surface(self) -> WlSurface {
        WlSurface::new(self.id)
    }
}

impl Decode for CreateSurface {
    type Output<'a> = Self;

    fn decode<'a>(reader: &mut Reader<'a>) -> Result<Self::Output<'a>, WlError> {
        Ok(Self { id: reader.read()? })
    }
}
