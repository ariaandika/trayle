// ===== Fixed =====

/// Wayland fixed primitive types.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Fixed(i32);

impl Fixed {
    #[inline]
    pub fn from_i32(int: i32) -> Self {
        Self(int)
    }

    #[inline]
    pub fn from_f32(float: f32) -> Self {
        Self((float * 256.0).round() as i32)
    }

    #[inline]
    pub fn to_i32(self) -> i32 {
        self.0
    }

    #[inline]
    pub fn to_f32(self) -> f32 {
        self.0 as f32 / 256.0
    }
}

impl std::fmt::Debug for Fixed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Fixed").field(&self.to_f32()).finish()
    }
}

impl std::fmt::Display for Fixed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_f32().fmt(f)
    }
}

// ===== Version =====

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

/// An operation version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Version(std::num::NonZeroU16);

impl Version {
    /// Create new [`Version`].
    ///
    /// Returns `None` if version is `0`.
    #[inline]
    pub const fn new(version: u32) -> Option<Self> {
        // perhaps reducing `u32` to `u16` is premature optimization, but its mentally better to
        // know that if its stored in a list, it does not force alignment 4
        match std::num::NonZeroU16::new(version as u16) {
            Some(ok) => Some(Self(ok)),
            None => None,
        }
    }

    pub const ONE: Self = Self(std::num::NonZeroU16::new(1).unwrap());

    /// Returns `Version` as `u32`.
    #[inline]
    pub const fn to_u32(self) -> u32 {
        self.0.get() as u32
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
        self.0.fmt(f)
    }
}
