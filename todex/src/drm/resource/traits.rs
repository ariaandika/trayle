use std::error;
use std::os::fd::AsFd;

use crate::drm::Handle;
use crate::drm::resource::ObjectType;

/// A type that represent a DRM resource.
///
/// Resource include connector, CRTC, encoder, framebuffer, and plane.
pub trait Resource: Sized {
    /// The error that can occur during resource request.
    type Error: error::Error;

    /// Resource [`ObjectType`].
    const OBJECT_TYPE: ObjectType;

    /// Request the resource for given handle.
    fn request<D: AsFd>(handle: Handle<Self>, device: &D) -> Result<Self, Self::Error>;
}
