//! Wayland protocol.
//!
//! This module provide API for wayland primitive, object abstraction, interface definitions,
//! message wire format.
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
//! # Other Traits
//!
//! This module also provide traits that abstract objects and messages.
//!
//! The following traits describe a whole instance:
//!
//! - [`WlObject`]: Type that is a wayland object.
//! - [`WlMessage`]: Type that is a wayland message
//!
//! These traits are not meant to be implemented by application.

pub use primitives::ObjectId;
pub use object::{Object, WlObject};
pub use interface::{AsInterface, WlInterface};
pub use message::{Message, WlMessage};
pub use error::WlError;

macro_rules! roundup4 {
    ($e:expr) => {
        ($e + 3) & (u16::MAX << 2)
    };
}

pub mod primitives;
pub mod object;
pub mod display;
pub mod interface;
pub mod message;
pub mod wire;
pub mod error;

mod prelude {
    pub use super::primitives::*;
    pub use super::object::{Object, WlGlobal};
    pub use super::wire::{AsOpCode, OpCode};
    pub use super::wire::{Decode, DecodeError, Decoder};
    pub use super::wire::{EncodePayload, Sized2, Writer};
    pub use super::message::{Message, WlMessage};
    pub use super::interface::AsInterface;
    pub use super::Interface;
    pub use super::display;
    pub use macros::{Interface, Message, OpCode, WlEnum, bitfield};
    pub use macros::interface;
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
