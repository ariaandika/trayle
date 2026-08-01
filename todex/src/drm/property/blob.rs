use crate::drm::Handle;
use crate::drm::ioctl::*;

#[derive(Debug)]
pub enum Blob {}

impl Blob {
    pub(crate) fn create<T>(data: &T, fd: BorrowedFd) -> Result<Handle<Self>, ErrCode> {
        let mut io = drm_mode_create_blob {
            data: data as *const T as _,
            length: size_of::<T>() as _,
            blob_id: None,
        };
        io.ioctl(fd)?;
        io.blob_id.ok_or_else(ErrCode::errno)
    }

    pub(crate) fn destroy(handle: Handle<Blob>, fd: BorrowedFd) -> Result<(), ErrCode> {
        drm_mode_destroy_blob { blob_id: handle }.ioctl(fd)
    }
}

// ===== syscall =====

#[repr(C)]
struct drm_mode_create_blob {
    data: __u64,
    length: __u32,
    blob_id: Option<Handle<Blob>>,
}

#[repr(C)]
struct drm_mode_destroy_blob {
    blob_id: Handle<Blob>,
}

impl DrmIoctl for drm_mode_create_blob {
    /// DRM_IOCTL_MODE_CREATEPROPBLOB
    const CODE: u32 = 0xBD;
}

impl DrmIoctl for drm_mode_destroy_blob {
    /// DRM_IOCTL_MODE_DESTROYPROPBLOB
    const CODE: u32 = 0xBE;
}
