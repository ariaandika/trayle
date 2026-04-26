//! # Todex
//!
//! Wayland protocol decoder and encoder.
mod ty;

pub mod encode;
pub mod proto;

// ===== reexports =====

pub use ty::{Id, Array, NewId};
