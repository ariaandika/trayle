//! # Todex
//!
//! Wayland protocol decoder and encoder.

mod primitive;

pub mod conn;
pub mod error;

pub mod message;
#[allow(warnings)]
pub mod wayland;

// ===== reexports =====

pub use primitive::{Id, Array, NewId};
