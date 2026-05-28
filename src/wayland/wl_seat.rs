use crate::seat::Capability;
use crate::wayland::prelude::*;
use crate::wayland::wl_keyboard::Keyboard;

// ===== op =====

pub enum RequestOp {
    GetPointer,
    GetKeyboard,
}

impl FromOp for RequestOp {
    fn from_op(op:u16) -> Result<Self,WlError>{
        match op {
            0 => Ok(Self::GetPointer),
            1 => Ok(Self::GetKeyboard),
            _ => Err(WlError::UnknownOp),
        }
    }
}

// ===== GetKeyboard =====

#[derive(Debug)]
pub struct GetKeyboard {
    id: Id,
}

impl GetKeyboard {
    pub fn keyboard(self) -> Keyboard {
        Keyboard::new(self.id)
    }
}

impl Decode for GetKeyboard {
    type Output<'a> = Self;

    fn decode(decoder: Decoder) -> Result<Self, WlError> {
        Ok(Self {
            id: decoder.read()?,
        })
    }
}

// ===== Capabilities =====

const EVENT_CAPABILITIES: u16 = 0;

pub fn capabilities(wl_seat: Id, capability: Capability, write: &mut Buffer) {
    unsafe { write.message(wl_seat, EVENT_CAPABILITIES, 12).put(capability.to_u32()) };
}
