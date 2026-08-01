pub(crate) use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
pub(crate) use std::ffi::{c_void, CStr};
pub(crate) use libc::{__u16, __u32, __u64, Ioctl};
pub(crate) use crate::sys::error::*;

// ===== private =====

pub(crate) enum IoKind {
    Io,
    #[expect(dead_code)]
    IoR,
    IoW,
    IoWR,
}

impl IoKind {
    const fn build<T>(self, code: u32) -> Ioctl {
        match self {
            Self::Io => libc::_IO(DRM_IOCTL_BASE, code),
            Self::IoR => libc::_IOR::<T>(DRM_IOCTL_BASE, code),
            Self::IoW => libc::_IOW::<T>(DRM_IOCTL_BASE, code),
            Self::IoWR => libc::_IOWR::<T>(DRM_IOCTL_BASE, code),
        }
    }
}

/// A type that is passed as an argument in drm `ioctl` calls.
pub(crate) trait DrmIoctl: Sized {
    /// The code will be translated to ioctl opcode automatically.
    const CODE: u32;

    /// `ioctl` opcode kind, the default is [`IoKind::IoWR`].
    const IO: IoKind = IoKind::IoWR;

    /// `ioctl` opcode.
    ///
    /// Implementor should not overwrite this. Change the value of [`DrmIoctl::IO`] instead.
    const OPCODE: Ioctl = Self::IO.build::<Self>(Self::CODE);

    /// Call `ioctl` with this type as argument.
    #[inline]
    fn ioctl(&mut self, fd: BorrowedFd) -> Result<(), ErrCode> {
        drm_ioctl(fd, Self::OPCODE, self as *mut Self as _)
    }
}

/// A type that only represent an opcode.
///
/// Most of the time, one struct have one associated opcode. But there is a case where two opcode
/// uses the same argument.
pub(crate) trait DrmOpCode<Payload>: Sized {
    /// The code will be translated to ioctl opcode automatically.
    const CODE: u32;

    /// `ioctl` opcode kind, the default is [`IoKind::IoWR`].
    const IO: IoKind = IoKind::IoWR;

    /// `ioctl` opcode.
    ///
    /// Implementor should not overwrite this. Change the value of [`DrmIoctl::IO`] instead.
    const OPCODE: Ioctl = Self::IO.build::<Payload>(Self::CODE);

    /// Call `ioctl` with given argument.
    #[inline]
    fn ioctl<T>(fd: BorrowedFd, arg: &mut T) -> Result<(), ErrCode> {
        drm_ioctl(fd, Self::OPCODE, arg as *mut T as _)
    }

    /// Call `ioctl` without an argument.
    #[inline]
    fn ioctl_null(fd: BorrowedFd) -> Result<(), ErrCode> {
        drm_ioctl(fd, Self::OPCODE, 0 as _)
    }
}

// ===== helpers =====

pub(super) union PadU32<T: Copy> {
    pub(super) pad: __u32,
    pub(super) value: T,
}

impl<T: Copy> From<T> for PadU32<T> {
    fn from(value: T) -> Self {
        Self { value }
    }
}

impl<T: Copy> Default for PadU32<T> {
    fn default() -> Self {
        Self { pad: 0 }
    }
}

// ===== syscall =====

// https://gitlab.freedesktop.org/mesa/libdrm

const DRM_IOCTL_BASE: u32 = 'd' as u32;

/// Call `ioctl`, restarting if it is interrupted.
fn drm_ioctl(fd: BorrowedFd, request: Ioctl, arg: *mut c_void) -> Result<(), ErrCode> {
    loop {
        let res = unsafe { libc::ioctl(fd.as_raw_fd(), request, arg) };
        if res != -1 {
            break;
        }
        let errno = ErrCode::errno();
        if !matches!(errno.code(), libc::EINTR | libc::EAGAIN) {
            return Err(errno);
        }
    }
    Ok(())
}
