pub use object_id::{AsObjectId, FromObjectId, NewId, ObjectId};
pub use op::OpCode;
pub use message::{Frame, Message, WaylandObject};
pub use buffer::{MessageBuf, SmallBuf};
pub use error::WlError;
pub use decode::Decode;
pub use encode::Encode;
pub use interface::Interface;

mod object_id;
mod op;
mod message;
mod buffer;
mod error;
mod decode;
mod encode;

pub mod interface;

pub mod wl_display;
pub mod wl_registry;
pub mod wl_callback;
pub mod wl_compositor;
pub mod wl_shm;
pub mod wl_data_source;
pub mod wl_data_device;
pub mod wl_data_device_manager;
pub mod wl_surface;
pub mod wl_seat;
pub mod wl_keyboard;

mod prelude {
    pub use super::object_id::{AsObjectId, FromObjectId, NewId, ObjectId};
    pub use super::op::OpCode;
    pub use super::message::{Message, WaylandObject};
    pub use super::error::WlError;
    pub use super::decode::{Decode, Decoder};
    pub use super::encode::{Encode, Encoder, WaylandEnum};
    pub use super::interface::Interface;

    pub(super) use super::op::opcode;
    pub(super) use super::message::simple_object;
    pub(super) use super::encode::encode_me;
}

macro_rules! roundup4 {
    ($e:expr) => {
        ($e + 3) & (u16::MAX << 2)
    };
}

use roundup4;
