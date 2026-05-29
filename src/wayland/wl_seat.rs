use crate::seat::Capability;
use crate::wayland::prelude::*;
use crate::wayland::wl_keyboard::Keyboard;

pub struct WlSeat {
    id: Id,
}

impl Object for WlSeat {
    const INTERFACE_ID: InterfaceId = InterfaceId::WlSeat;

    #[inline]
    fn id(&self) -> Id {
        self.id
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

pub fn capabilities(wl_seat: Id, capabilities: Capability) -> Message<Capabilities> {
    Message::new(&WlSeat { id: wl_seat }, Capabilities { capabilities })
}

pub struct Capabilities {
    capabilities: Capability,
}

impl Encode for Message<Capabilities> {
    fn encode(self, encoder: Encoder) {
        encoder.encode_one(self.id(), EventOp::Capabities as u16, self.capabilities.to_u32());
    }
}
