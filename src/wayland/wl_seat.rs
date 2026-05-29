use crate::seat::Capability;
use crate::wayland::prelude::*;
use crate::wayland::wl_keyboard::Keyboard;

pub struct Seat {
    id: Id,
}

impl FromId for Seat {
    fn from_id(id: Id) -> Self {
        Self { id }
    }
}

impl Object for Seat {
    const INTERFACE_ID: InterfaceId = InterfaceId::WlSeat;

    #[inline]
    fn id(&self) -> Id {
        self.id
    }
}

impl Seat {
    pub fn capabilities(&self, capabilities: Capability) -> Message<Capabilities> {
        Message::new(self, Capabilities { capabilities })
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

#[derive(Debug, Clone, Copy)]
pub enum EventOp {
    Capabities,
}

impl ToOp for EventOp {
    fn to_op(&self) -> u16 {
        *self as u16
    }
}

// ===== GetKeyboard =====

#[derive(Debug)]
pub struct GetKeyboard {
    pub keyboard: NewId<Keyboard>,
}

impl Decode for GetKeyboard {
    type Output<'a> = Self;

    fn decode(decoder: Decoder) -> Result<Self, WlError> {
        Ok(Self {
            keyboard: decoder.read()?,
        })
    }
}

// ===== Capabilities =====

pub struct Capabilities {
    capabilities: Capability,
}

impl Encode for Message<Capabilities> {
    fn encode(self, encoder: Encoder) {
        encoder.encode_one(self.id(), EventOp::Capabities as u16, self.capabilities.to_u32());
    }
}
