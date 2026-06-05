pub use object_id::{AsObjectId, FromObjectId, NewId, ObjectId};
pub use op::OpCode;
pub use object::WlObject;
pub use message::{Frame, Message};
pub use buffer::{MessageBuf, SmallBuf};
pub use error::WlError;
pub use decode::Decode;
pub use encode::Encode;
pub use interface::{AsInterface, Interface};

mod object_id;
mod op;
mod object;
mod error;
mod message;
mod decode;
mod encode;

pub mod buffer;
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
    pub use super::message::Message;
    pub use super::error::WlError;
    pub use super::decode::{Decode, Decoder};
    pub use super::encode::{Encode, Encoder, WaylandEnum};
    pub use super::interface::{AsInterface, Interface};

    pub use macros::{Interface, Message, OpCode};
}

macro_rules! roundup4 {
    ($e:expr) => {
        ($e + 3) & (u16::MAX << 2)
    };
}

use roundup4;
