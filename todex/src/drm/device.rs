use crate::drm::ioctl::*;
use crate::drm::resource::Resources;
use crate::drm::property::Blob;
use crate::drm::capability::ClientCapability;
use crate::drm::{Framebuffer, Handle, master, Plane};

pub trait Device: AsFd {
    /// Get available connector, CRTC, encoder and framebuffer handles.
    #[inline]
    fn get_resources(&self) -> Result<Resources, ErrCode> {
        Resources::get_resources(self.as_fd())
    }

    /// Get plane handles.
    #[inline]
    fn get_plane_handles(&self) -> Result<Box<[Handle<Plane>]>, ErrCode> {
        Plane::get_handles(self.as_fd())
    }

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
