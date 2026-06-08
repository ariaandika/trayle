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

impl WlEnum for Capability {
    fn from_u32(uint: u32) -> Option<Self> {
        Some(Self::from_u32(uint))
    }

    fn to_u32(self) -> u32 {
        self.to_u32()
    }
}
