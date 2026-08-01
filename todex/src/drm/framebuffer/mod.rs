use crate::drm::ioctl::*;
use crate::drm::Handle;
use crate::drm::resource::{Resource, ObjectType};
use crate::fourcc::Format;

/// DRM Framebuffer.
///
/// Framebuffers are abstract memory objects that provide a source of pixel data to scanout to a
/// CRTC. Applications explicitly request the creation of framebuffers and can control their be‐
/// havior. Framebuffers rely on the underneath memory manager for low-level memory operations. When
/// creating a framebuffer, applications pass a memory handle through the API which is used as
/// backing storage. The framebuffer itself is only an abstract object with no data.
#[derive(Debug)]
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub format: Format,
    pub flags: u32,
    /// Driver specific handle.
    pub handles: [u32; 4],
    pub strides: [u32; 4],
    pub offsets: [u32; 4],
    pub modifiers: [u64; 4],
}

impl Framebuffer {
    pub(crate) fn add_fb2(&self, fd: BorrowedFd) -> Result<Handle<Framebuffer>, ErrCode> {
        let mut io = drm_mode_fb_cmd2 {
            fb_id: None,
            width: self.width,
            height: self.height,
            pixel_format: self.format,
            flags: self.flags,
            handles: self.handles,
            pitches: self.strides,
            offsets: self.offsets,
            modifier: self.modifiers,
        };
        AddFb2::ioctl(fd, &mut io)?;
        io.fb_id
            // .ok_or_else(|| Error::custom("kernel returns invalid id"))
            .ok_or_else(|| todo!("errno"))
    }

    fn get_resource(handle: Handle<Self>, fd: BorrowedFd) -> Result<Self, ErrCode> {
        let mut io = drm_mode_fb_cmd2 {
            fb_id: Some(handle),
            width: 0,
            height: 0,
            pixel_format: Format::XRGB8888,
            flags: 0,
            handles: [0; 4],
            pitches: [0; 4],
            offsets: [0; 4],
            modifier: [0; 4],
        };
        GetFb2::ioctl(fd, &mut io)?;
        Ok(Self {
            width: io.width,
            height: io.height,
            format: io.pixel_format,
            flags: io.flags,
            handles: io.handles,
            strides: io.pitches,
            offsets: io.offsets,
            modifiers: io.modifier,
        })
    }
}

impl Resource for Framebuffer {
    type Error = ErrCode;

    const OBJECT_TYPE: ObjectType = ObjectType::FB;

    #[inline]
    fn request<D: AsFd>(handle: Handle<Self>, device: &D) -> Result<Self, Self::Error> {
        Self::get_resource(handle, device.as_fd())
    }
}

// ===== syscalls =====

#[derive(Debug)]
#[repr(C)]
struct drm_mode_fb_cmd2 {
    /// Object ID of the framebuffer.
    fb_id: Option<Handle<Framebuffer>>,
    /// Width of the framebuffer.
    width: __u32,
    /// Height of the framebuffer.
    height: __u32,
    /// FourCC format code, see ``DRM_FORMAT_*`` constants in ``drm_fourcc.h``.
    pixel_format: Format,
    /// Framebuffer flags (see &DRM_MODE_FB_INTERLACED and &DRM_MODE_FB_MODIFIERS).
    flags: __u32,

    /// GEM buffer handle, one per plane. Set to 0 if the plane is unused. The same handle can be
    /// used for multiple planes.
    handles: [__u32; 4],
    /// @pitches: Pitch (aka. stride) in bytes, one per plane.
    pitches: [__u32; 4],
    /// @offsets: Offset into the buffer in bytes, one per plane.
    offsets: [__u32; 4],
    /// Format modifier, one per plane. See ``DRM_FORMAT_MOD_*`` constants in
    /// ``drm_fourcc.h``. All planes must use the same modifier. Ignored unless
    /// &DRM_MODE_FB_MODIFIERS is set in @flags.
    modifier: [__u64; 4],
}

struct AddFb2;

impl DrmOpCode<drm_mode_fb_cmd2> for AddFb2 {
    /// DRM_IOCTL_MODE_ADDFB2
    const CODE: u32 = 0xB8;
}

struct GetFb2;

impl DrmOpCode<drm_mode_fb_cmd2> for GetFb2 {
    /// DRM_IOCTL_MODE_GETFB2
    const CODE: u32 = 0xCE;
}
