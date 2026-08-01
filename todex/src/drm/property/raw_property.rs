use crate::drm::ioctl::*;

// ===== RawProperty =====

/// Pair of property id and raw value.
#[derive(Debug, Clone, Copy)]
pub struct RawProperty {
    pub id: u32,
    pub value: u64,
}

impl RawProperty {
    pub fn get_named_property<D: AsFd>(&self, device: &D) -> Result<NamedRawProperty, ErrCode> {
        self.get_inner(device.as_fd())
    }

    pub(super) fn get_inner(&self, fd: BorrowedFd) -> Result<NamedRawProperty, ErrCode> {
        let mut io = drm_mode_get_property {
            prop_id: self.id,
            ..<_>::default()
        };
        io.ioctl(fd)?;
        Ok(NamedRawProperty {
            id: self.id,
            value: self.value,
            name: unsafe { CStr::from_ptr(io.name.as_ptr().cast()).into() },
        })
    }
}

// ===== NamedRawProperty =====

#[derive(Debug)]
pub struct NamedRawProperty {
    pub id: u32,
    pub value: u64,
    pub name: Box<CStr>,
}

// ===== syscall =====

const DRM_PROP_NAME_LEN: usize = 32;

#[derive(Default)]
#[repr(C)]
struct drm_mode_get_property {
    values_ptr: __u64,
    enum_blob_ptr: __u64,
    /// set by the caller.
    prop_id: __u32,
    /// DRM_MODE_PROP_*`` bitfield. See &drm_property.flags
    flags: __u32,
    name: [u8; DRM_PROP_NAME_LEN],
    count_values: __u32,
    count_enum_blobs: __u32,
}

impl DrmIoctl for drm_mode_get_property {
    /// DRM_IOCTL_MODE_GETPROPERTY
    const CODE: u32 = 0xAA;
}
