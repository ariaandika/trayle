use crate::drm::ioctl::*;
use crate::drm::Handle;
use crate::drm::resource::ResourceError;
use crate::drm::resource::{ObjectType, Resource};

/// DRM Encoder.
///
/// An encoder takes pixel data from a CRTC and converts it to a format suitable for any attached
/// connectors. On some devices, it may be possible to have a CRTC send data to more than one en‐
/// coder. In that case, both encoders would receive data from the same scanout buffer, resulting in
/// a cloned display configuration across the connectors attached to each encoder.
#[derive(Debug)]
pub struct Encoder {
    pub handle: Handle<Self>,
}

impl Resource for Encoder {
    type Error = ResourceError;

    const OBJECT_TYPE: ObjectType = ObjectType::ENCODER;

    #[inline]
    fn request<D: AsFd>(handle: Handle<Self>, device: &D) -> Result<Self, Self::Error> {
        drm_mode_get_encoder {
            encoder_id: handle.into(),
            ..<_>::default()
        }
        .ioctl(device.as_fd())
        .map_or_else(|e| Err(e.into()), |()| Ok(Self { handle }))
    }
}

// ===== EncoderType =====

#[derive(Debug, Clone, Copy)]
pub enum EncoderType {
    None = 0,
    DAC = 1,
    TMDS = 2,
    LVDS = 3,
    TVDAC = 4,
    VIRTUAL = 5,
    DSI = 6,
    DPMST = 7,
    DPI = 8,
}

// ===== syscall =====

#[derive(Default)]
#[repr(C)]
struct drm_mode_get_encoder {
    encoder_id: __u32,
    encoder_type: PadU32<EncoderType>,
    crtc_id: __u32,
    possible_crtcs: __u32,
    possible_clones: __u32,
}

impl DrmIoctl for drm_mode_get_encoder {
    /// DRM_IOCTL_MODE_GETENCODER
    const CODE: u32 = 0xA6;
}
