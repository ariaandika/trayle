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

opcode! {
    pub enum RequestOp {
        GetPointer,
        GetKeyboard,
    }
}

opcode! {
    pub enum EventOp {
        Capabilities,
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
        encoder.encode_one(self.id(), EventOp::Capabilities, self.capabilities.to_u32());
    }
}
