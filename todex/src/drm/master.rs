use std::ffi::c_int;

use crate::drm::ioctl::*;

pub(crate) fn auth_magic(magic: drm_magic_t, fd: BorrowedFd) -> Result<(), ErrCode> {
    drm_auth { magic }.ioctl(fd)
}

pub(crate) fn set_master(fd: BorrowedFd) -> Result<(), ErrCode> {
    drm_set_master::ioctl_null(fd)
}

pub(crate) fn is_master(fd: BorrowedFd<'_>) -> bool {
    // Detect master by attempting something that requires master.
    //
    // Authenticating magic tokens requires master and 0 is an
    // internal kernel detail which we could use. Attempting this on
    // a master fd would fail therefore fail with EINVAL because 0
    // is invalid.
    //
    // A non-master fd will fail with EACCES, as the kernel checks
    // for master before attempting to do anything else.
    //
    // Since we don't want to leak implementation details, use
    // EACCES.
    //
    // source: libdrm/xf86drm.c:3268
    match auth_magic(0, fd) {
        Ok(_) => true, // 0 != EACCES
        Err(_) => ErrCode::raw_errno() != libc::EACCES,
    }
}

pub(crate) fn drop_master(fd: BorrowedFd) -> Result<(), ErrCode> {
    drm_drop_master::ioctl_null(fd)
}

#[expect(non_camel_case_types)]
type drm_magic_t = c_int;

#[repr(C)]
struct drm_set_master;

#[repr(C)]
struct drm_drop_master;

#[derive(Default)]
#[repr(C)]
struct drm_auth {
    magic: drm_magic_t,
}

impl DrmOpCode<Self> for drm_set_master {
    const CODE: u32 = 0x1e;

    const IO: IoKind = IoKind::Io;
}

impl DrmOpCode<Self> for drm_drop_master {
    /// DRM_IOCTL_DROP_MASTER
    const CODE: u32 = 0x1f;

    const IO: IoKind = IoKind::Io;
}

impl DrmIoctl for drm_auth {
    /// DRM_IOCTL_AUTH_MAGIC
    const CODE: u32 = 0x11;

    const IO: IoKind = IoKind::IoW;
}
