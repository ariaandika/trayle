//! Wayland protocol.
//!
//! This module provide wayland primitive, object abstraction, interface definitions, message decoding and
//! encoding.
//!
//! # Primitives
//!
//! Most wayland primitives represented by rust primitive:
//!
//! - `int` `uint` -> `i32` `u32`
//! - `fixed` -> [`Fixed`]
//! - `string` -> `&str`
//! - `array` -> `&[u8]`
//! - `fd` -> `i32`
//!
//! [`ObjectId`] represent wayland object id. Object id cannot be `0`.
//!
//! [`NewId<I>`] wraps object id with type safe interface, representing new id for new object that
//! implement associated interface. Note that implicit interface for new id is not supported, its
//! embedded as field in the message. Its massively increase complexity while in practice only
//! **one** message uses it.
//!
//! `object` in wayland represented as type safe [`Object<I>`] with associated interface. It can
//! also represent untyped object where the interface stored as runtime value [`Interface`].
//!
//! The `fd` are pulled from ancillary data.
//!
//! # Object / Interface
//!
//! Wayland is an object-oriented protocol. Each object follows exactly one interface. An interface
//! is a collection of message and enumeration definitions.
//!
//! To create an object, select one of available interface and use [`FromObjectId`] implementation.
//! Each interface have its message's constructor as method. All of the methods returns message
//! payload wrapped in [`Message`] to associate it with an object id.
//!
//! [`Object`] can also be untyped, the interface is stored as runtime value [`Interface`]. This can
//! be used to store object in generic collection. In this case, object cannot create message but
//! still have the common property of an object.
//!
//! Wayland enum represented as regular enum. Bitfield enum represented as struct wrapper of `u32`.
//!
//! # Decoding / Encoding
//!
//! This API use [`Bytes`] and [`Cmsg`] for memory management. It is a bytes buffer and fds storage.
//! See its documentation for more details.
//!
//! [`Frame`] is used to decode a message. [`Frame::has_frame`] returns `true` if the buffer
//! contains enough bytes for a frame. Then the buffer can be passed to `Frame` to decode the actual
//! message using [`Decode`] implementation.
//!
//! To encode a message, use the [`Encode`] implementation for corresponding message.
//!
//! This crate have a convention where every interface definition have a method to construct its
//! messages wrapped in a [`Message`] to associate it with an object id. With this, application can
//! use [`EncodeMessage`] to encode a message directly without passing object id around.
//!
//! # Other Traits
//!
//! This module also provide traits that abstract objects and messages.
//!
//! The following traits describe associated property of a type:
//!
//! - [`AsObjectId`]: Type that is associated with an object id.
//! - [`AsInterface`]: Type that is associated with an interface.
//! - [`AsOpCode`]: Type that is associated with an opcode.
//!
//! The following traits describe a whole instance:
//!
//! - [`WlObject`]: Type that is a wayland object.
//! - [`WlGlobal`]: Type that is a singleton global object.
//! - [`WlMessage`]: Type that is a wayland message
//! - [`OpCode`]: Type that is a request/event opcode
//!
//! These traits are not meant to be implemented by application.

// `ObjectData` and `display` are not documented currently

// ===== core components =====

pub use object_id::{AsNewId, AsObjectId, FromObjectId, NewId, ObjectId};
pub use types::{Fixed, Version};
pub use interface::AsInterface;
pub use opcode::{AsOpCode, OpCode};
pub use enums::WlEnum;
pub use object::{Any, Object, ObjectError, WlObject};
pub use message::{Message, WlMessage};

// ===== properties =====

pub use global::{WlGlobal, Global};
pub use constructor::Constructor;
pub use operation::Operation;
pub use error::WlError;

// ===== decode/encode =====

pub use frame::{Frame, FrameError};
pub use decode::{Decode, DecodeError};
pub use encode::{Encode, EncodeMessage};

macro_rules! roundup4 {
    ($e:expr) => {
        ($e + 3) & (u16::MAX << 2)
    };
}

mod object_id;
mod types;
mod interface;
mod opcode;
mod enums;
mod object;
mod message;

mod global;
mod constructor;
mod operation;
mod error;

mod frame;
mod decode;
mod encode;

pub mod handle;
pub mod display;

mod prelude {
    pub use super::{FromObjectId, AsObjectId, AsNewId, AsInterface, AsOpCode};
    pub use super::{WlGlobal, OpCode, WlMessage, WlEnum};
    pub use super::{Fixed, Interface, Message, NewId, Object, ObjectId, Version};
    pub use super::decode::{Decode, Decoder, DecodeError};
    pub use super::encode::{Encode, Sized2, Writer};
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
