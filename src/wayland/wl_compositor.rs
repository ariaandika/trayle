use crate::wayland::prelude::*;
use crate::wayland::wl_surface::WlSurface;

// ===== Op =====

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
    pub surface: NewId<WlSurface>,
}

impl Decode for CreateSurface {
    type Output<'a> = Self;

    fn decode(decoder: Decoder<'_>) -> Result<Self::Output<'_>, WlError> {
        Ok(Self {
            surface: decoder.read()?,
        })
    }
}
