use crate::drm::ioctl::*;
use crate::drm::resource::{ResourceError, Resources};
use crate::drm::property::Blob;
use crate::drm::capability::ClientCapability;
use crate::drm::{Framebuffer, Handle, master, Plane};

/// The DRM Device.
///
/// This trait provide resource queries, framebuffer management, capability query and setting, DRM
/// authentication, and blob property management as default methods.
pub trait Device: AsFd {
    /// Request available connectors, CRTCs, encoders and framebuffers.
    #[inline]
    fn resources(&self) -> Result<Resources, ResourceError> {
        Resources::get_resources(self.as_fd())
    }

    /// Request available planes as handles.
    #[inline]
    fn planes(&self) -> Result<Box<[Handle<Plane>]>, ResourceError> {
        Plane::get_resource(self.as_fd())
    }

    /// Create blob property, returns the handle to it.
    #[inline]
    fn create_property_blob<T>(&self, data: &T) -> Result<Handle<Blob>, ErrCode> {
        Blob::create(data, self.as_fd())
    }

    #[inline]
    fn destroy_blob(&self, handle: Handle<Blob>) -> Result<(), ErrCode> {
        Blob::destroy(handle, self.as_fd())
    }

    #[inline]
    fn add_framebuffer(&self, fb: &Framebuffer) -> Result<Handle<Framebuffer>, ErrCode> {
        fb.add_fb2(self.as_fd())
    }

    /// Set client capability.
    #[inline]
    fn set_client_capability(&self, capability: ClientCapability, value: bool) -> Result<(), ErrCode> {
        capability.set_capability(value, &self.as_fd())
    }

    #[inline]
    fn set_master(&self) -> Result<(), ErrCode> {
        master::set_master(self.as_fd())
    }

    #[inline]
    fn is_master(&self) -> bool {
        master::is_master(self.as_fd())
    }

    #[inline]
    fn drop_master(&self) -> Result<(), ErrCode> {
        master::drop_master(self.as_fd())
    }
}
