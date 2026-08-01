use crate::drm::ioctl::*;
use crate::drm::{Crtc, Framebuffer, Handle};
use crate::drm::resource::{ObjectType, Resource};
use crate::fourcc::Format;

#[derive(Debug)]
pub struct Plane {
    /// This plane handle.
    pub handle: Handle<Plane>,
    /// Current CRTC handle.
    pub crtc: Option<Handle<Crtc>>,
    /// Current framebuffer handle.
    pub fb: Option<Handle<Framebuffer>>,
    /// Bitmask of CRTC's compatible with the plane.
    pub possible_crtcs: u32,
    /// Supported formats.
    pub formats: Box<[Format]>,
}

impl Plane {
    pub(crate) fn get_handles(fd: BorrowedFd) -> Result<Box<[Handle<Plane>]>, ErrCode> {
        let mut io = drm_mode_get_plane_res::default();
        io.ioctl(fd)?;
        let mut plane_ids = Box::new_uninit_slice(io.count_planes as _);
        io.plane_id_ptr = plane_ids.as_mut_ptr() as _;
        io.ioctl(fd)?;
        Ok(unsafe { plane_ids.assume_init() })
    }

    fn get_resource(handle: Handle<Self>, fd: BorrowedFd) -> Result<Self, ErrCode> {
        let mut io = drm_mode_get_plane {
            plane_id: Some(handle),
            ..<_>::default()
        };
        io.ioctl(fd)?;
        let mut formats = Box::new_uninit_slice(io.count_format_types as _);
        io.format_type_ptr = formats.as_mut_ptr() as _;
        io.ioctl(fd)?;
        Ok(Self {
            handle,
            crtc: io.crtc_id,
            fb: io.fb_id,
            possible_crtcs: io.possible_crtcs,
            formats: unsafe { formats.assume_init() },
        })
    }
}

impl Resource for Plane {
    type Error = ErrCode;

    const OBJECT_TYPE: ObjectType = ObjectType::PLANE;

    #[inline]
    fn get_resource<D: AsFd>(handle: Handle<Self>, device: &D) -> Result<Self, Self::Error> {
        Self::get_resource(handle, device.as_fd())
    }
}

// ===== PlaneType =====

/// Plane type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaneType {
    Overlay,
    Primary,
    Cursor,
}

impl From<PlaneType> for u64 {
    #[inline]
    fn from(value: PlaneType) -> Self {
        value as u64
    }
}

// ===== syscall =====

#[derive(Default)]
#[repr(C)]
struct drm_mode_get_plane_res {
    plane_id_ptr: __u64,
    count_planes: __u32,
}

#[derive(Default)]
#[repr(C)]
struct drm_mode_get_plane {
    plane_id: Option<Handle<Plane>>,
    crtc_id: Option<Handle<Crtc>>,
    fb_id: Option<Handle<Framebuffer>>,
    /// possible_crtcs: bitmask of CRTC's compatible index with the plane
    possible_crtcs: __u32,
    /// Never used
    gamma_size: __u32,
    count_format_types: __u32,
    format_type_ptr: __u64,
}

impl DrmIoctl for drm_mode_get_plane_res {
    /// DRM_IOCTL_MODE_GETPLANERESOURCES
    const CODE: u32 = 0xB5;
}

impl DrmIoctl for drm_mode_get_plane {
    /// DRM_IOCTL_MODE_GETPLANE
    const CODE: u32 = 0xB6;
}
