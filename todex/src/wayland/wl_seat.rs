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
    GetTouch,
    Release,
}

#[derive(Message, Debug)]
#[request(WlSeat)]
pub struct GetKeyboard {
    pub keyboard: NewId<WlKeyboard>,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Capabilities,
    Name,
}

#[derive(Message, Debug)]
#[event(WlSeat)]
pub struct Capabilities {
    pub capabilities: Capability,
}

#[derive(Debug, Clone, Copy)]
pub struct Capability(u32);

bitfield! {
    Capability;
    Pointer = 1,
    Keyboard = 2,
    Touch = 4,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    /// get_pointer, get_keyboard or get_touch called on seat without the matching capability
    MissingCapability,
}
