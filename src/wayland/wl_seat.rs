use crate::wayland::prelude::*;
use crate::wayland::wl_keyboard::Keyboard;
use crate::wayland::wl_pointer::Pointer;

// ===== capability =====

const POINTER: u32 = 1;
const KEYBOARD: u32 = 1 << 1;
// const TOUCH: u32 = 1 << 2;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Capability(u32);

impl Capability {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn add_pointer(self) -> Self {
        Self(self.0 | POINTER)
    }

    pub const fn add_keyboard(self) -> Self {
        Self(self.0 | KEYBOARD)
    }
}

const EVENT_CAPABILITIES: u16 = 0;

impl Capability {
    pub fn encode(self, wl_seat: Id, write: &mut Buffer) {
        unsafe { write.message(wl_seat, EVENT_CAPABILITIES, 12).put(self.0) };
    }
}

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

// ===== GetPointer =====

#[derive(Debug)]
pub struct GetPointer {
    id: Id,
}

impl GetPointer {
    pub fn pointer(self) -> Pointer {
        Pointer::new(self.id)
    }
}

impl Decode for GetPointer {
    type Output<'a> = Self;

    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        Ok(Self {
            id: reader.read()?,
        })
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

    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        Ok(Self {
            id: reader.read()?,
        })
    }
}

