use crate::wayland::prelude::*;
use crate::wayland::wl_surface::Surface;

simple_object! {
    pub struct WlCompositor::Compositor;
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
    pub surface: NewId<Surface>,
}

impl Decode for CreateSurface {
    type Output<'a> = Self;

    fn decode(decoder: Decoder<'_>) -> Result<Self::Output<'_>, WlError> {
        Ok(Self {
            surface: decoder.read()?,
        })
    }
}
