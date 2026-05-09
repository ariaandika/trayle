//! # Todex
//!
//! Wayland protocol decoder and encoder.

// ===== standard =====

mod net;

// ===== type definitions =====

mod primitive;

pub mod message;
pub mod error;

pub use primitive::{Id, Array, NewId};

// ===== logic =====

mod object_manager;
pub mod conn;

// ===== generated code =====
#[allow(warnings)]
pub mod wayland;

