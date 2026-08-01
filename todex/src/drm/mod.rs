//! DRM Kernel Mode-Setting API.
//!
//! # Objects
//!
//! The following are 5 KMS objects:
//!
//! - [`CRTC`][Crtc], part of the chip that contains a pointer to a scanout buffer.
//! - [`Plane`], image source that can be blended with or overlayed on top of a CRTC during scanout.
//! - [`Encoder`], takes pixel data from a CRTC and converts it to a format suitable for connectors.
//! - [`Connector`], the final destination of pixel-data on a device.
//! - [`Framebuffer`], memory objects that provide a source of pixel data to scanout to a CRTC.
//!
//! # Usage
//!
//! Before mode-setting can be performed, an application needs to call [`Device::set_master`] to
//! become DRM-Master. It then has exclusive access to the KMS API. A call to
//! [`Device::get_resources`] returns a list of CRTCs, Connectors, Encoders and Planes.

// ===== reexports =====

pub use handle::Handle;
pub use plane::Plane;
pub use connector::Connector;
pub use crtc::Crtc;
pub use framebuffer::Framebuffer;
pub use encoder::Encoder;
pub use device::Device;

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
