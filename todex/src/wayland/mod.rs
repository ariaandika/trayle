//! Wayland protocol.
//!
//! # Usage
//!
//! This API use [`MessageBuf`] as memory management. `MessageBuf` is a bytes buffer that can also
//! stores fds. Application can establish unix socket externally, then use [`MessageBuf::sendmsg`]
//! or [`MessageBuf::recvmsg`] to send and receive messages respectively.
//!
//! [`MessageBuf::has_frame`] returns `true` if the buffer contains enough bytes for a frame. Then
//! it can be passed to [`Frame`] to decode the actual message. Application can send back a message,
//! using [`Encode`] trait. Note that this operation is buffered, application requires to flush the
//! message using `MessageBuf::sendmsg` mentioned previously.
//!
//! # Types
//!
//! [`ObjectId`] represents wayland object id. `ObjectId` cannot be zero. [`NewId`] is a wrapper for
//! `ObjectId` with generic parameter to represent created object. Other primitive types can be
//! represented by its respective rust primitive types.
//!
//! [`Interface`] is a runtime value representing an interface. This can be used by high level APIs
//! to store mutliple interfaces in a list without dynamic dispatch.
//!
//! [`Encodable`] associate object id to a message payload. Encoding required its interface object
//! id. One cannot simply define object id field to a message payload. This struct wraps the payload
//! and associate it with object id to form a complete encodable message.
//!
//! # Error
//!
//! All fallible operations returns `Result` with [`WlError`] as the error variant.
//!
//! # Traits
//!
//! This module also provide traits that can be used by high level APIs:
//!
//! - [`FromObjectId`]: Constructs type with given object id.
//! - [`AsObjectId`]: Type that is associated with an object id.
//! - [`OpCode`]: Request/event opcode
//! - [`AsInterface`]: Type that is belong to an interface.
//! - [`WlObject`]: Type that represent a wayland object.
//!
//! # Interfaces
//!
//! Interface definitions are provided in the module with the same name of the interface. All
//! respective types implements all traits mentioned previously.
//!
//! Every interface module follows a convention.
//!
//! - Object definitions is the UpperCamelCase of the interface name.
//! - `RequestOp` and `EventOp` representing requests and events of the interface.
//! - Operation definition are regular struct.
//! - Interface object contains constructor methods for its operations.

pub use object_id::{AsObjectId, FromObjectId, NewId, ObjectId};
pub use fixed::Fixed;
pub use error::WlError;
pub use traits::{AsInterface, AsOpCode, OpCode, WlEnum, WlObject};
pub use message::Frame;

pub use decode::Decode;
pub use encode::{Encodable, Encode, EncodeMessage};

macro_rules! roundup4 {
    ($e:expr) => {
        ($e + 3) & (u16::MAX << 2)
    };
}

mod object_id;
mod fixed;
mod error;
mod traits;
mod message;
mod decode;
mod encode;

mod prelude {
    pub use super::{AsObjectId, FromObjectId};
    pub use super::{AsInterface, AsOpCode, Decode, Encode, OpCode, WlEnum};
    pub use super::{Fixed, Interface, NewId, ObjectId, WlError};
    pub use super::decode::Decoder;
    pub use super::encode::{Encodable, Sized2, Writer};

    pub use macros::{Interface, Message, OpCode, WlEnum, bitfield};
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
    pub mod wl_shm_pool;
    pub mod wl_shm;
    pub mod wl_buffer;
    pub mod wl_data_offer;
    pub mod wl_data_source;
    pub mod wl_data_device;
    pub mod wl_data_device_manager;
    pub mod wl_surface;
    pub mod wl_seat;
    pub mod wl_pointer;
    pub mod wl_keyboard;
    pub mod wl_touch;
    pub mod wl_output;
    pub mod wl_region;
    pub mod wl_subcompositor;
    pub mod wl_subsurface;
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
