use crate::compositor::seat::Capability;
use crate::wayland::prelude::*;
use crate::wayland::wl_keyboard::WlKeyboard;

simple_object! {
    /// `wl_seat` interface.
    pub struct WlSeat;
}

impl WlSeat {
    /// Create `wl_seat::get_keyboard` request.
    #[inline]
    pub fn get_keyboard(&self, keyboard: NewId<WlKeyboard>) -> Message<GetKeyboard> {
        Message::new(self, GetKeyboard { keyboard })
    }

    /// Create `wl_seat::capabilities` event.
    #[inline]
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

/// `wl_seat::get_keyboard` request.
#[derive(Debug)]
pub struct GetKeyboard {
    pub keyboard: NewId<WlKeyboard>,
}

impl Decode for GetKeyboard {
    type Output<'a> = Self;

    #[inline]
    fn decode(decoder: Decoder) -> Result<Self, WlError> {
        Ok(Self {
            keyboard: decoder.read()?,
        })
    }
}

impl Encode for Message<GetKeyboard> {
    const OPCODE: u16 = RequestOp::GetKeyboard as u16;

    #[inline]
    fn object_id(&self) -> ObjectId {
        self.object_id()
    }

    #[inline]
    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.keyboard);
    }
}

// ===== Capabilities =====

/// `wl_seat::capabilities` event.
pub struct Capabilities {
    capabilities: Capability,
}

impl Decode for Capabilities {
    type Output<'a> = Self;

    #[inline]
    fn decode<'a>(decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError> {
        Ok(Self { capabilities: Capability::from_u32(decoder.read()?)  })
    }
}

impl Encode for Message<Capabilities> {
    const OPCODE: u16 = EventOp::Capabilities as u16;

    #[inline]
    fn object_id(&self) -> ObjectId {
        self.object_id()
    }

    #[inline]
    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.capabilities.to_u32());
    }
}
