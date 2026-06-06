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

// ===== impls =====

impl super::decode::Read<'_> for KeymapFormat {
    #[inline]
    fn decode(reader: &mut super::decode::Reader<'_>) -> Result<Self, WlError> {
        match reader.read::<u32>()? as u8 {
            0 => Ok(Self::NoKeymap),
            1 => Ok(Self::XkbV1),
            _ => Err(WlError::UnknownObject),
        }
    }
}

impl super::encode::Write for KeymapFormat {
    #[inline]
    fn size(&self) -> u16 {
        4
    }

    #[inline]
    fn encode(self, writer: &mut super::encode::Writer) {
        (self as u32).encode(writer);
    }
}
