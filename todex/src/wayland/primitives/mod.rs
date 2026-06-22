//! Primitive types
//!
//! Most wayland primitives represented by rust primitive.
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
//! The following traits describe associated property of a type:
//!
//! - [`AsObjectId`]: Type that is associated with an object id.

pub use object_id::{AsNewId, AsObjectId, FromObjectId, NewId, ObjectId};
pub use types::{Fixed, Version};

mod object_id;
mod types;

// ===== Enum =====

/// Type that represent a wayland enum.
pub trait WlEnum: Sized {
    /// Create enum from integer.
    ///
    /// Returns `None` if the integer did not represent valid entry.
    fn from_u32(uint: u32) -> Option<Self>;

    /// Returns `u32` representation of the enum.
    fn to_u32(self) -> u32;
}
