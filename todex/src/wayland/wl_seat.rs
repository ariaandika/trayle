use crate::wayland::prelude::*;
use crate::wayland::wl_keyboard::WlKeyboard;
use crate::wayland::wl_pointer::WlPointer;
use crate::wayland::wl_touch::WlTouch;

#[derive(Interface, Debug)]
#[interface(global = 10)]
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
#[message(request = WlSeat)]
pub struct GetPointer {
    pub pointer: NewId<WlPointer>,
}

#[derive(Message, Debug)]
#[message(request = WlSeat)]
pub struct GetKeyboard {
    pub keyboard: NewId<WlKeyboard>,
}

#[derive(Message, Debug)]
#[message(request = WlSeat)]
pub struct GetTouch {
    pub touch: NewId<WlTouch>,
}

#[derive(Message, Debug)]
#[message(request = WlSeat, destructor)]
pub struct Release;

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Capabilities,
    Name,
}

#[derive(Message, Debug)]
#[message(event = WlSeat)]
pub struct Capabilities {
    pub capabilities: Capability,
}

#[derive(Message, Debug)]
#[message(event = WlSeat, since = 2)]
pub struct Name<'a> {
    pub name: &'a str,
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

impl Error {
    pub fn message(&self) -> &'static str {
        match self {
            Error::MissingCapability => "missing seat capability",
        }
    }
}
