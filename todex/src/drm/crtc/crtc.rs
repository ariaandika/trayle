use crate::drm::ioctl::*;
use crate::drm::Handle;
use crate::drm::resource::ResourceError;
use crate::drm::resource::{ObjectType, Resource};
use crate::drm::connector::ModeInfo;

/// DRM CRTC.
///
/// A CRTC short for CRT Controller is an abstraction representing a part of the chip that con‐
/// tains a pointer to a scanout buffer. Therefore, the number of CRTCs available determines how
/// many independent scanout buffers can be active at any given time. The CRTC structure contains
/// several fields to support this: a pointer to some video memory (abstracted as a frame-buffer
/// object), a list of driven connectors, a display mode and an (x, y) offset into the video mem‐
/// ory to support panning or configurations where one piece of video memory spans multiple CRTCs. A
/// CRTC is the central point where configuration of displays happens.
#[derive(Debug)]
pub struct Crtc {
    /// This CRTC handle.
    pub handle: Handle<Self>,
    /// Framebuffer x.
    pub x: u32,
    /// Framebuffer y.
    pub y: u32,
    /// Mode width.
    pub width: u32,
    /// Mode height.
    pub height: u32,
}

impl Crtc {
    fn get_resource(handle: Handle<Self>, device: BorrowedFd) -> Result<Self, ResourceError> {
        let mut io = drm_mode_crtc {
            crtc_id: handle.into(),
            ..<_>::default()
        };
        io.ioctl(device)?;
        if io.mode_valid == 0 {
            io.mode.hdisplay = 0;
            io.mode.vdisplay = 0;
        }
        Ok(Self {
            handle,
            x: io.x,
            y: io.y,
            width: io.mode.hdisplay as _,
            height: io.mode.vdisplay as _,
        })
    }
}

impl Resource for Crtc {
    type Error = ResourceError;

    const OBJECT_TYPE: ObjectType = ObjectType::CRTC;

    #[inline]
    fn request<D: AsFd>(handle: Handle<Self>, device: &D) -> Result<Self, Self::Error> {
        Self::get_resource(handle, device.as_fd())
    }
}

// ===== syscalls =====

#[derive(Default)]
#[repr(C)]
struct drm_mode_crtc {
    set_connectors_ptr: __u64,
    count_connectors: __u32,
    crtc_id: __u32,
    fb_id: __u32,
    // framebuffer position
    x: __u32,
    y: __u32,
    gamma_size: __u32,
    mode_valid: __u32,
    mode: ModeInfo,
}

impl DrmIoctl for drm_mode_crtc {
    /// DRM_IOCTL_MODE_GETCRTC
    const CODE: u32 = 0xA1;
}
