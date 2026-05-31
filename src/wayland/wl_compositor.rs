use crate::wayland::prelude::*;
use crate::wayland::wl_surface::WlSurface;

simple_object! {
    pub struct WlCompositor;
}

// ===== Op =====

opcode! {
    pub enum RequestOp {
        CreateSurface,
    }
}

// ===== CreateSurace =====

#[derive(Debug)]
pub struct CreateSurface {
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
