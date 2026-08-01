use crate::drm::ioctl::*;
use crate::drm::{Connector, Crtc, Encoder, Framebuffer, Handle};

/// Resource handles, returned from [`Device::resources`].
///
/// [`Device::resources`]: crate::drm::Device::resources
#[derive(Debug)]
pub struct Resources {
    pub fbs: Box<[Handle<Framebuffer>]>,
    pub crtcs: Box<[Handle<Crtc>]>,
    pub connectors: Box<[Handle<Connector>]>,
    pub encoders: Box<[Handle<Encoder>]>,
    pub min_width: u32,
    pub max_width: u32,
    pub min_height: u32,
    pub max_height: u32,
}

impl Resources {
    pub(crate) fn get_resources(fd: BorrowedFd) -> Result<Self, ErrCode> {
        let mut io = drm_mode_card_res::default();
        io.ioctl(fd)?;
        let mut fbs = Box::new_uninit_slice(io.count_fbs as _);
        let mut crtcs = Box::new_uninit_slice(io.count_crtcs as _);
        let mut connectors = Box::new_uninit_slice(io.count_connectors as _);
        let mut encoders = Box::new_uninit_slice(io.count_encoders as _);
        io.fb_id_ptr = fbs.as_mut_ptr() as _;
        io.crtc_id_ptr = crtcs.as_mut_ptr() as _;
        io.connector_id_ptr = connectors.as_mut_ptr() as _;
        io.encoder_id_ptr = encoders.as_mut_ptr() as _;
        io.ioctl(fd)?;
        unsafe {
            Ok(Self {
                fbs: fbs.assume_init(),
                crtcs: crtcs.assume_init(),
                connectors: connectors.assume_init(),
                encoders: encoders.assume_init(),
                min_width: io.min_width,
                max_width: io.max_width,
                min_height: io.min_height,
                max_height: io.max_height,
            })
        }
    }
}

// ===== syscall =====

#[derive(Debug, Default)]
#[repr(C)]
struct drm_mode_card_res {
    fb_id_ptr: __u64,
    crtc_id_ptr: __u64,
    connector_id_ptr: __u64,
    encoder_id_ptr: __u64,
    count_fbs: __u32,
    count_crtcs: __u32,
    count_connectors: __u32,
    count_encoders: __u32,
    min_width: __u32,
    max_width: __u32,
    min_height: __u32,
    max_height: __u32,
}

impl DrmIoctl for drm_mode_card_res {
    /// DRM_IOCTL_MODE_GETRESOURCES
    const CODE: u32 = 0xA0;
}
