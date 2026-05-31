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
    const OPCODE: u16 = EventOp::Capabilities as u16;

    #[inline]
    fn object_id(&self) -> Id {
        self.id()
    }

    #[inline]
    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.capabilities.to_u32());
    }
}
