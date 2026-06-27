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

pub use object_id::{AsObjectId, ObjectId};
pub use types::{Fixed, Version};

mod object_id;
mod types;

// ===== Enum =====

/// Type that represent a wayland enum.
///
/// # Protocol Violation
///
/// Note that so far, there is no practical difference between using `int` and `uint` to represent
/// an `enum`. Therefore, any args with an enum of `int`, will be casted to `u32` before transformed
/// into the enum type, and vice versa.
pub trait WlEnum: Sized {
    /// Create enum from `uint`.
    ///
    /// Returns `None` if the integer did not represent valid entry.
    fn from_u32(uint: u32) -> Option<Self>;

    /// Returns `uint` representation of the enum.
    fn to_u32(self) -> u32;

    /// Create enum from `int`.
    ///
    /// The default implementation is to passed the int to [`WlEnum::from_u32`]. See the trait docs
    /// for more detail.
    #[inline]
    fn from_i32(int: i32) -> Option<Self> {
        Self::from_u32(int as u32)
    }

    /// Create enum from `int`.
    ///
    /// The default implementation is to cast the result of [`WlEnum::to_u32`]. See the trait docs
    /// for more detail.
    #[inline]
    fn to_i32(self) -> i32 {
        self.to_u32() as i32
    }
}
