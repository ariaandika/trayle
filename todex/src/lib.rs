//! # Todex
//!
//! Wayland protocol decoder and encoder.

mod net;

mod primitive;

pub mod conn;
pub mod error;

pub mod message;

// generated code
#[allow(warnings)]
pub mod wayland;

// ===== reexports =====

pub use primitive::{Id, Array, NewId};
