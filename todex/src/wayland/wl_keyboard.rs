use crate::wayland::prelude::*;

#[derive(Interface, Debug)]
pub struct WlKeyboard {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Keymap,
}

#[derive(Message, Debug)]
#[event(WlKeyboard)]
pub struct Keymap {
    pub format: KeymapFormat,
    #[fd]
    pub fd: i32,
    pub size: u32,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum KeymapFormat {
    NoKeymap,
    XkbV1
}
