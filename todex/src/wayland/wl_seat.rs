use crate::compositor::seat::Capability;
use crate::wayland::prelude::*;
use crate::wayland::wl_keyboard::WlKeyboard;

#[derive(Interface, Debug)]
pub struct WlSeat {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    GetPointer,
    GetKeyboard,
}

#[derive(Message, Debug)]
#[request(WlSeat)]
pub struct GetKeyboard {
    pub keyboard: NewId<WlKeyboard>,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Capabilities,
}

#[derive(Message, Debug)]
#[event(WlSeat)]
pub struct Capabilities {
    pub capabilities: Capability,
}

// ===== impls =====

impl super::decode::Read<'_> for Capability {
    #[inline]
    fn decode(reader: &mut super::decode::Reader<'_>) -> Result<Self, WlError> {
        Ok(Self::from_u32(reader.read()?))
    }
}

impl WlEnum for Capability {
    fn to_u32(self) -> u32 {
        self.to_u32()
    }
}
