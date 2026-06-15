//! Wayland protocol.
//!
//! # Usage
//!
//! This API use [`Buffer`] and [`Cmsg`] as memory management. It is a bytes buffer and fds storage.
//! Application can establish unix socket externally, then use [`Cmsg::sendmsg`] and
//! [`Cmsg::recvmsg`] to send and receive messages respectively.
//!
//! [`Frame::has_frame`] returns `true` if the buffer contains enough bytes for a frame. Then the
//! buffer can be passed to [`Frame`] to decode the actual message. Application can send back a
//! message, using [`Encode`] trait. Note that this operation is buffered, application requires to
//! flush the message using method mentioned previously.
//!
//! [`Buffer`]: crate::sys::buffer::Buffer
//! [`Cmsg`]: crate::sys::cmsg::Cmsg
//! [`Cmsg::sendmsg`]: crate::sys::cmsg::Cmsg::sendmsg
//! [`Cmsg::recvmsg`]: crate::sys::cmsg::Cmsg::recvmsg
//!
//! # Types
//!
//! [`ObjectId`] represents wayland object id. `ObjectId` cannot be zero. [`NewId`] is a wrapper for
//! `ObjectId` with generic parameter to represent created object.
//!
//! [`Fixed`] represents wayland fixed primitive. Can be created from `f32`.
//!
//! Other primitive types can be represented by its respective rust primitive types.
//!
//! [`Interface`] is a runtime value representing an interface. This can be used by high level APIs
//! to store mutliple interfaces in a list without dynamic dispatch.
//!
//! [`Object`] represent wayland object. It can be type safe or runtime value.
//!
//! [`Encodable`] associate object id to a message payload. Encoding required its interface object
//! id. One cannot simply define object id field to a message payload. This struct wraps the payload
//! and associate it with object id to form a complete encodable message.
//!
//! # Enum
//!
//! Wayland enum represented as regular enum. Bitfield enum represented as struct wrapper of `u32`.
//!
//! # Error
//!
//! All fallible operations returns `Result` with [`WlError`] as the error variant.
//!
//! # Traits
//!
//! This module also provide traits that can be used by high level APIs.
//!
//! - [`FromObjectId`]: Constructs type with given object id.
//! - [`AsOpCode`]: Type that is associated with an opcode.
//! - [`AsObjectId`]: Type that is associated with an object id.
//! - [`AsInterface`]: Type that is associated with an interface.
//! - [`OpCode`]: Request/event opcode
//! - [`WlObject`]: Type that represent a wayland object.
//!
//! These traits are not meant to be implemented by application.
//!
//! # Interface
//!
//! Interface definitions are provided in the module with the same name of the interface. All
//! respective types implements all traits mentioned previously.
//!
//! Every interface module follows a convention, with some exception for `wl_display`.
//!
//! - Object definitions is the UpperCamelCase of the interface name.
//! - `RequestOp` and `EventOp` representing requests and events of the interface.
//! - Operation definition are regular struct.
//! - Interface object contains constructor methods for its operations.
//!
//! For example with `wl_registry`:
//! - Interface object: `wl_registry::WlRegistry`
//! - Request opcodes: `wl_registry::RequestOp`
//! - Event opcodes: `wl_registry::EventOp`
//! - `wl_registry::bind` request: `wl_registry::Bind`
//! - `wl_registry::global` event: `wl_registry::Global`
//! - `wl_registry::bind` constructor: `wl_registry::WlRegistry::bind`
//! - `wl_registry::global` constructor: `wl_registry::WlRegistry::global`

pub use object_id::{AsObjectId, FromObjectId, NewId, ObjectId};
pub use fixed::Fixed;
pub use object::{Object, Any, ObjectError};
pub use error::WlError;
pub use traits::{AsInterface, AsOpCode, OpCode, WlEnum, WlObject};
pub use message::{Frame, MessageError};

pub use decode::{Decode, DecodeError};
pub use encode::{Encodable, Encode, EncodeMessage};

macro_rules! roundup4 {
    ($e:expr) => {
        ($e + 3) & (u16::MAX << 2)
    };
}

mod object_id;
mod fixed;
mod object;
mod error;
mod traits;
mod message;
mod decode;
mod encode;

pub mod display;

mod prelude {
    pub use super::{AsObjectId, FromObjectId};
    pub use super::{AsInterface, AsOpCode, Decode, Encode, OpCode, WlEnum};
    pub use super::{Fixed, Interface, NewId, Object, ObjectId};
    pub use super::decode::{Decoder, DecodeError};
    pub use super::encode::{Encodable, Sized2, Writer};
    pub use super::display;

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
    pub mod xdg_wm_base;
    pub mod xdg_positioner;
    pub mod xdg_surface;
    pub mod xdg_toplevel;
    pub mod xdg_popup;
    #[todo] pub mod zwp_linux_dmabuf_v1;
    #[todo] pub mod zwp_linux_buffer_params_v1;
    #[todo] pub mod zwp_linux_dmabuf_feedback_v1;
}
