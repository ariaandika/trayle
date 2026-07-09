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

macro_rules! roundup4 {
    ($e:expr) => {
        ($e + 3) & (u16::MAX << 2)
    };
}

pub mod error;
pub mod primitives;
pub mod object;
pub mod message;
pub mod interface;
pub mod wire;
pub mod display;

#[cfg(doc)]
pub mod cheatsheet {
    //! Cheatsheet
    //!
    //! # Common
    //!
    //! APIs that used in compositor logic.
    //!
    //! ## Traits
    //!
    //! - [`AsObjectId`], [`AsVersion`], [`AsNewId`], [`AsInterface`]: associated data
    //! - [`WlMessage`]: constant props for message (`AsInterface` + `AsOpCode`)
    //! - [`WlInterface`]:
    //! - [`InterfaceMarker`]:
    //!
    //! ## Wrappers
    //!
    //! - [`ObjectId`], [`Version`], [`NewId`]
    //!
    //! ## Shared struct
    //!
    //! - [`Object`]
    //! - [`Message`]
    //!
    //! # Encoding
    //!
    //! APIs that used in decoding/encoding.
    //!
    //! - [`AsOpCode`], [`OpCode`]
    //! - [`Payload`]
    //! - [`DecodePayload`], [`Encode`]
    //!
    use super::primitives::*;
    use super::object::*;
    use super::message::*;
    use super::interface::*;
    use super::wire::*;
    use super::display::*;
    use super::error::*;
}
