use std::error;
use std::os::fd::AsFd;

use crate::drm::Handle;
use crate::drm::resource::ObjectType;
use crate::sys::error::ErrCode;

/// A type that represent a drm resource.
///
/// Resource include connector, crtc, encoder, framebuffer, and plane.
pub trait Resource: Sized {
    type Error: error::Error;

    const OBJECT_TYPE: ObjectType;

    /// Fetch the resource by handle.
    fn get_resource<D: AsFd>(handle: Handle<Self>, device: &D) -> Result<Self, ErrCode>;
}
