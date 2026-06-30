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

pub use object::Object;
pub use message::{Message, WlMessage};
pub use interface::{AsInterface, Interface};
pub use error::WlError;

macro_rules! roundup4 {
    ($e:expr) => {
        ($e + 3) & (u16::MAX << 2)
    };
}

pub mod primitives;
pub mod object;
pub mod message;
pub mod interface;
pub mod wire;
pub mod display;
pub mod error;
