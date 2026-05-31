use crate::compositor::seat::Capability;
use crate::wayland::prelude::*;
use crate::wayland::wl_keyboard::WlKeyboard;

simple_object! {
    pub struct WlSeat;
}

impl WlSeat {
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
    pub keyboard: NewId<WlKeyboard>,
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
