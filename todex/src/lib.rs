//! # Todex
//!
//! Wayland protocol decoder and encoder.

mod primitive;

pub mod message;
pub mod error;

mod wayland;

// ===== reexports =====

pub use primitive::{Id, Array, NewId};
