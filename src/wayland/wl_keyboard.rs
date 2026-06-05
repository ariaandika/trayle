use crate::wayland::prelude::*;

#[derive(Debug, Interface)]
pub struct WlKeyboard {
    id: ObjectId,
}

impl WlKeyboard {
    /// Create `wl_keyboard::keymap` event.
    #[inline]
    pub fn keymap(&self, format: KeymapFormat, fd: i32, size: u32) -> Message<Keymap> {
        Message::new(self, Keymap { format, fd, size })
    }
}

// ===== Op =====

opcode! {
    pub enum EventOp {
        Keymap,
    }
}

// ===== Keymap =====

/// `wl_keyboard::keymap` event.
#[derive(Debug)]
pub struct Keymap {
    format: KeymapFormat,
    fd: i32,
    size: u32,
}

impl Decode for Keymap {
    type Output<'a> = Self;

    #[inline]
    fn decode<'a>(mut decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError> {
        let fd = decoder.pop_fd()?;
        let mut reader = decoder.reader();
        Ok(Self {
            format: reader.read()?,
            fd,
            size: reader.read()?,
        })
    }
}

impl Encode for Message<Keymap> {
    const OPCODE: u16 = EventOp::Keymap as u16;

    #[inline]
    fn encode(self, mut encoder: Encoder) {
        encoder.push_fd(self.fd);
        encode_me!(encoder, self, format, size);
    }
}

// ===== KeymapFormat =====

/// `wl_keyboard::keymap_format` enum.
#[derive(Debug, Clone, Copy)]
pub enum KeymapFormat {
    NoKeymap,
    XkbV1
}

impl<'a> super::decode::Read<'a> for KeymapFormat {
    #[inline]
    fn decode(reader: &mut super::decode::Reader<'a>) -> Result<Self, WlError> {
        match reader.read()? {
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
