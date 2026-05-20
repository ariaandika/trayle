pub use id::Id;
pub use error::WlError;

mod id;
mod error;
mod decode;
mod encode;

pub mod wl_display;
pub mod wl_registry;

mod prelude {
    pub use super::id::Id;
    pub use super::error::WlError;
    pub use super::decode::{Decoder, Decode};
    pub use super::encode::{PtrWrite, Encoder};
    pub use crate::buffer::Buffer;

    pub(super) use super::roundup4;
}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
#[allow(clippy::enum_variant_names, reason = "for now just wl_*")]
pub enum Interface {
    WlDisplay,
    WlRegistry,
    WlCallback,
}

/// `(id, op, len)`
pub fn header(bytes: &[u8]) -> Option<(u32, u16, u16, &[u8])> {
    let (header, rest) = bytes.split_first_chunk::<8>()?;
    let ptr = header.as_ptr();
    unsafe {
        let id = u32::from_ne_bytes(*ptr.cast::<[u8; _]>());
        let op = u16::from_ne_bytes(*ptr.add(4).cast::<[u8; _]>());
        let len = u16::from_ne_bytes(*ptr.add(6).cast::<[u8; _]>());
        let body_len = len.saturating_sub(8) as usize;
        Some((id, op, len, rest.get(..body_len)?))
    }
}

macro_rules! roundup4 {
    ($e:expr) => {
        ($e + 3) & (u16::MAX << 2)
    };
}

use roundup4;
