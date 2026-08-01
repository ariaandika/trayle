use std::{fmt, ops};

use crate::bitflags::{Bitflags, simple_bitflags, simple_bitflags_debug};
use crate::drm::ioctl::*;

// ===== ModeInfo =====

const DRM_DISPLAY_MODE_LEN: usize = 32;

/// Display mode information.
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct ModeInfo {
    /// Pixel clock in kHz.
    pub clock: __u32,
    /// Horizontal display size.
    pub hdisplay: __u16,
    /// Horizontal sync start.
    pub hsync_start: __u16,
    /// Horizontal sync end.
    pub hsync_end: __u16,
    /// Horizontal total size.
    pub htotal: __u16,
    /// Horizontal skew.
    pub hskew: __u16,
    /// Vertical display size.
    pub vdisplay: __u16,
    /// Vertical sync start.
    pub vsync_start: __u16,
    /// Vertical sync end.
    pub vsync_end: __u16,
    /// Vertical total size.
    pub vtotal: __u16,
    /// Vertical scan.
    pub vscan: __u16,
    /// Approximate vertical refresh rate in Hz.
    pub vrefresh: __u32,
    /// Bitmask of misc. flags, see DRM_MODE_FLAG_* defines.
    pub flags: __u32,
    /// Bitmask of type flags, see DRM_MODE_TYPE_* defines.
    pub type_: ModeType,
    /// String describing the mode resolution.
    pub name: ModeName,
}

// ===== ModeName =====

/// String describing the [`ModeInfo`] resolution.
#[derive(Default, Clone)]
#[repr(C)]
pub struct ModeName {
    /// Guarantee to ends with null termination.
    bytes: [u8; DRM_DISPLAY_MODE_LEN],
}

impl ModeName {
    /// Returns the name as [`CStr`].
    ///
    /// This method calls [`CStr::from_bytes_until_nul`], which performs null search, so its better
    /// to cache this method result.
    #[inline]
    pub fn as_cstr(&self) -> &CStr {
        // SAFETY: guarantee to ends with null termination.
        unsafe { CStr::from_bytes_until_nul(&self.bytes).unwrap_unchecked() }
    }
}

impl ops::Deref for ModeName {
    type Target = CStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_cstr()
    }
}

impl fmt::Debug for ModeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_cstr().fmt(f)
    }
}

// ===== ModeType =====

/// Mode type bitflags.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ModeType(u32);

simple_bitflags!(ModeType, u32);
simple_bitflags_debug!(
    ModeType, BUILTIN, CLOCK_C, CRTC_C, PREFERRED, DEFAULT, USERDEF, DRIVER, ALL
);

// source: libdrm/include/drm/drm_mode.h

impl ModeType {
    /// Deprecated.
    pub const BUILTIN: Self = Self(1 << 0);
    /// Deprecated.
    pub const CLOCK_C: Self = Self(1 << 1 | Self::BUILTIN.0);
    /// Deprecated.
    pub const CRTC_C: Self = Self(1 << 2 | Self::BUILTIN.0);
    pub const PREFERRED: Self = Self(1 << 3);
    /// Deprecated.
    pub const DEFAULT: Self = Self(1 << 4);
    pub const USERDEF: Self = Self(1 << 5);
    pub const DRIVER: Self = Self(1 << 6);
    pub const ALL: Self = Self(Self::PREFERRED.0 | Self::USERDEF.0 | Self::DRIVER.0);

    /// Returns `true` if the type is [`ModeType::PREFERRED`].
    #[inline]
    pub fn is_preferred(self) -> bool {
        self.contains(Self::PREFERRED)
    }
}
