pub use object_id::{AsObjectId, FromObjectId, NewId, ObjectId};
pub use message::{Frame, Message};
pub use buffer::{MessageBuf, SmallBuf};
pub use error::WlError;
pub use decode::Decode;
pub use encode::Encode;
pub use traits::{OpCode, WlObject, AsInterface};

macro_rules! roundup4 {
    ($e:expr) => {
        ($e + 3) & (u16::MAX << 2)
    };
}

mod object_id;
mod error;
mod message;
mod decode;
mod encode;
mod traits;

pub mod buffer;

mod prelude {
    pub use super::object_id::{AsObjectId, FromObjectId, NewId, ObjectId};
    pub use super::message::Message;
    pub use super::error::WlError;
    pub use super::decode::{Decode, Decoder};
    pub use super::encode::{Encode, Encoder, WaylandEnum};
    pub use super::traits::{OpCode, AsInterface};
    pub use super::Interface;

    pub use macros::{Interface, Message, OpCode};
}

macros::protocol! {
    /// Reexport interfaces as upper camel case.
    pub mod interfaces;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Interface;

    pub mod wl_display;
    pub mod wl_registry;
    pub mod wl_callback;
    pub mod wl_compositor;
    #[todo]
    pub mod wl_shm_pool;
    pub mod wl_shm;
    #[todo]
    pub mod wl_buffer;
    #[todo]
    pub mod wl_data_offer;
    pub mod wl_data_source;
    pub mod wl_data_device;
    pub mod wl_data_device_manager;
    pub mod wl_surface;
    pub mod wl_seat;
    #[todo]
    pub mod wl_pointer;
    pub mod wl_keyboard;
    #[todo] pub mod wl_touch;
    #[todo] pub mod wl_output;
    #[todo] pub mod wl_region;
    #[todo] pub mod wl_subcompositor;
    #[todo] pub mod wl_subsurface;
    #[todo] pub mod wl_fixes;
    #[todo] pub mod xdg_wm_base;
    #[todo] pub mod xdg_positioner;
    #[todo] pub mod xdg_surface;
    #[todo] pub mod xdg_toplevel;
    #[todo] pub mod xdg_popup;
    #[todo] pub mod zwp_linux_dmabuf_v1;
    #[todo] pub mod zwp_linux_buffer_params_v1;
    #[todo] pub mod zwp_linux_dmabuf_feedback_v1;
}
