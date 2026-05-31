pub use id::{Id, NewId};
pub use op::OpCode;
pub use message::{Frame, Message, Object};
pub use error::WlError;
pub use decode::Decode;
pub use encode::Encode;
pub use interface::InterfaceId;

mod id;
mod op;
mod message;
mod error;
mod decode;
mod encode;

mod interface;

pub mod wl_display;
pub mod wl_registry;
pub mod wl_callback;
pub mod wl_compositor;
pub mod wl_shm;
pub mod wl_data_device_manager;
pub mod wl_surface;
pub mod wl_seat;
pub mod wl_keyboard;

mod prelude {
    pub use super::id::{FromId, Id, NewId};
    pub use super::op::OpCode;
    pub use super::message::{Message, Object};
    pub use super::error::WlError;
    pub use super::decode::{Decode, Decoder};
    pub use super::encode::{Encode, Encoder};
    pub use super::interface::InterfaceId;

    pub(super) use super::roundup4;
    pub(super) use super::op::opcode;
    pub(super) use super::message::simple_object;
}

macro_rules! roundup4 {
    ($e:expr) => {
        ($e + 3) & (u16::MAX << 2)
    };
}

use roundup4;
