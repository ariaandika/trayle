use crate::wayland::prelude::*;
use crate::wayland::wl_surface::WlSurface;

// ===== Op =====

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    CreateSurface,
}

// ===== CreateSurace =====

#[derive(Debug)]
pub struct CreateSurface {
    pub surface: NewId<WlSurface>,
}

impl Decode for CreateSurface {
    type Output<'a> = Self;

    #[inline]
    fn decode(decoder: Decoder<'_>) -> Result<Self::Output<'_>, WlError> {
        Ok(Self {
            surface: decoder.read()?,
        })
    }
}

impl Encode for Message<CreateSurface> {
    const OPCODE: u16 = RequestOp::CreateSurface as u16;

    #[inline]
    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.surface);
    }
}
