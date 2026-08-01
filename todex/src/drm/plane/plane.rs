use crate::drm::ioctl::*;
use crate::drm::{Crtc, Framebuffer, Handle};
use crate::drm::resource::{ObjectType, Resource, ResourceError};
use crate::fourcc::Format;

/// DRM Plane.
///
/// A plane respresents an image source that can be blended with or overlayed on top of a CRTC
/// during the scanout process. Planes are associated with a frame-buffer to crop a portion of the
/// image memory (source) and optionally scale it to a destination size. The result is then blended
/// with or overlayed on top of a CRTC. Planes are not provided by all hardware and the number of
/// available planes is limited. If planes are not available or if not enough planes are available,
/// the user should fall back to normal software blending (via GPU or CPU).
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
    pub(crate) fn get_resource(fd: BorrowedFd) -> Result<Box<[Handle<Plane>]>, ResourceError> {
        let mut io = drm_mode_get_plane_res::default();
        io.ioctl(fd)?;
        let mut plane_ids = Box::new_uninit_slice(io.count_planes as _);
        io.plane_id_ptr = plane_ids.as_mut_ptr() as _;
        io.ioctl(fd)?;
        Ok(unsafe { plane_ids.assume_init() })
    }

    fn get_plane(handle: Handle<Self>, fd: BorrowedFd) -> Result<Self, ErrCode> {
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
    fn request<D: AsFd>(handle: Handle<Self>, device: &D) -> Result<Self, Self::Error> {
        Self::get_plane(handle, device.as_fd())
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
