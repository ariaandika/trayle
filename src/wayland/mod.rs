pub use id::Id;
pub use op::FromOp;
pub use error::WlError;
pub use decode::Decode;
pub use interface::InterfaceId;

mod id;
mod op;
mod error;
mod decode;
mod encode;

pub mod wl_display;
pub mod wl_registry;
pub mod wl_shm;
pub mod wl_data_device_manager;
pub mod wl_seat;
pub mod wl_pointer;
pub mod wl_keyboard;

mod interface;

mod prelude {
    pub use super::Object;
    pub use super::id::Id;
    pub use super::op::FromOp;
    pub use super::error::WlError;
    pub use super::decode::{Reader, Decode};
    pub use super::encode::{PtrWrite, Encoder};
    pub use super::interface::InterfaceId;
    pub use crate::buffer::Buffer;

    pub(super) use super::roundup4;
}

pub trait Object {
    const INTERFACE_ID: InterfaceId;

    fn id(&self) -> Id;
}

macro_rules! roundup4 {
    ($e:expr) => {
        ($e + 3) & (u16::MAX << 2)
    };
}

use roundup4;
