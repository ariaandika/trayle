use crate::bitflags::{Bitflags, simple_bitflags};
use crate::drm::ioctl::*;

// ===== ModeInfo =====

const DRM_DISPLAY_MODE_LEN: usize = 32;

#[derive(Default, Clone)]
#[repr(C)]
pub struct ModeInfo {
    pub clock: __u32,
    pub hdisplay: __u16,
    pub hsync_start: __u16,
    pub hsync_end: __u16,
    pub htotal: __u16,
    pub hskew: __u16,
    pub vdisplay: __u16,
    pub vsync_start: __u16,
    pub vsync_end: __u16,
    pub vtotal: __u16,
    pub vscan: __u16,

    pub vrefresh: __u32,

    pub flags: __u32,
    pub type_: ModeType,
    pub name: [u8; DRM_DISPLAY_MODE_LEN],
}

impl ModeInfo {
    #[inline]
    pub fn name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.name).unwrap()
    }
}

impl std::fmt::Debug for ModeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("drm_mode_modeinfo")
            .field("clock", &self.clock)
            .field("hdisplay", &self.hdisplay)
            .field("hsync_start", &self.hsync_start)
            .field("hsync_end", &self.hsync_end)
            .field("htotal", &self.htotal)
            .field("hskew", &self.hskew)
            .field("vdisplay", &self.vdisplay)
            .field("vsync_start", &self.vsync_start)
            .field("vsync_end", &self.vsync_end)
            .field("vtotal", &self.vtotal)
            .field("vscan", &self.vscan)
            .field("vrefresh", &self.vrefresh)
            .field("flags", &self.flags)
            .field("type", &self.type_)
            .field("name", &CStr::from_bytes_until_nul(&self.name))
            .finish()
    }
}

// ===== ModeType =====

/// Mode type bitflags.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ModeType(u32);

simple_bitflags!(ModeType, u32);

impl ModeType {
    pub const BUILTIN: Self = Self(1 << 0); /* deprecated */
    pub const CLOCK_C: Self = Self(1 << 1 | Self::BUILTIN.0); /* deprecated */
    pub const CRTC_C: Self = Self(1 << 2 | Self::BUILTIN.0); /* deprecated */
    pub const PREFERRED: Self = Self(1 << 3);
    pub const DEFAULT: Self = Self(1 << 4); /* deprecated */
    pub const USERDEF: Self = Self(1 << 5);
    pub const DRIVER: Self = Self(1 << 6);
    pub const ALL: Self = Self(Self::PREFERRED.0 | Self::USERDEF.0 | Self::DRIVER.0);

    #[inline]
    pub fn is_preferred(self) -> bool {
        self.contains(Self::PREFERRED)
    }
}
// macro_rules! impl_ops {
//     ($(impl $tr:ident::$fn:ident;)*) => {$(
//         impl std::ops::$tr for ModeType {
//             type Output = Self;
//
//             #[inline]
//             fn $fn(self, rhs: Self) -> Self::Output {
//                 Self(self.0.$fn(rhs.0))
//             }
//         }
//     )*};
// }
// impl_ops! {
//     impl BitOr::bitor;
//     impl BitAnd::bitand;
//     impl BitXor::bitxor;
// }

impl std::fmt::Debug for ModeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! entries {
            ($($entry:ident,)*) => { const { [
                $((Self::$entry,stringify!($entry)),)*
            ] } };
        }
        let entries = entries![
            BUILTIN, CLOCK_C, CRTC_C, PREFERRED, DEFAULT, USERDEF, DRIVER,
        ];
        let mut has_flag = false;
        f.write_str("ModeType(")?;
        for (mode, name) in entries {
            if !self.contains(mode) {
                continue;
            }
            if has_flag {
                f.write_str(" | ")?;
            }
            f.write_str(name)?;
            has_flag = true;
        }
        f.write_str(")")
    }
}

// ===== syscall =====

