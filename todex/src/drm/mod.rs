//! DRM Kernel Mode-Setting API.
//!
//! # Usage
//!
//! To begin using this API, the [`Device`] trait must be implemented. Pretty much top-level
//! functions from `DRM` is available as default methods in the trait.

// ===== reexports =====

pub use handle::Handle;
pub use plane::Plane;
pub use connector::Connector;
pub use crtc::Crtc;
pub use framebuffer::Framebuffer;
pub use encoder::Encoder;
pub use device::Device;

// TODO: resource: custom request error
// TODO: property: custom request error
// TODO: plane: possible crtc helper type

// ===== mods =====

mod ioctl;
mod handle;

pub mod property;
pub mod resource;

mod master;
pub mod capability;

pub mod plane;
pub mod connector;
pub mod crtc;
pub mod framebuffer;
pub mod encoder;

pub mod atomic;
pub mod event;

mod device;
