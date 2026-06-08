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

#[derive(Debug, Clone, Copy)]
pub enum KeymapFormat {
    NoKeymap,
    XkbV1
}

impl WlEnum for KeymapFormat {
    #[inline]
    fn from_u32(uint: u32) -> Option<Self> {
        match uint {
            0 => Some(Self::NoKeymap),
            1 => Some(Self::XkbV1),
            _ => None,
        }
    }

    #[inline]
    fn to_u32(self) -> u32 {
        self as u32
    }
}
