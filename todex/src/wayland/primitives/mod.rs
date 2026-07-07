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
pub use fixed::Fixed;
pub use version::{AsVersion, Version};
pub use traits::WlEnum;

mod object_id;
mod fixed;
mod version;
mod traits;
