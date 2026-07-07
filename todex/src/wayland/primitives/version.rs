// ===== AsVersion =====

/// Type that is associated with a version.
pub trait AsVersion {
    /// Returns the associated version.
    fn version(&self) -> Version;
}

impl<I: AsVersion> AsVersion for &I {
    #[inline]
    fn version(&self) -> Version {
        I::version(self)
    }
}

// ===== Version =====

/// An operation version.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(Inner);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[expect(unused)]
enum Inner {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    V10,
    V11,
    V12,
    V13,
    V14,
    V15,
}

const MAX: u8 = Inner::V15 as u8;

impl Version {
    /// Create new [`Version`].
    ///
    /// Returns `None` if version is unsupported.
    #[inline]
    pub const fn new(version: u32) -> Option<Self> {
        if version as u8 <= MAX {
            Some(Self(unsafe {
                core::mem::transmute::<u8, Inner>(version as u8)
            }))
        } else {
            None
        }
    }

    /// Version 1.
    pub const ONE: Self = Self(Inner::V1);

    /// Returns `Version` as `u32`.
    #[inline]
    pub const fn to_u32(self) -> u32 {
        self.0 as u32
    }
}

impl AsVersion for Version {
    #[inline]
    fn version(&self) -> Version {
        *self
    }
}

impl std::fmt::Display for Version {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.0 as u8).fmt(f)
    }
}

impl std::fmt::Debug for Version {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Version").field(&(self.0 as u8)).finish()
    }
}
