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

/// An operation version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Version(std::num::NonZeroU32);

impl Version {
    /// Create new [`Version`].
    ///
    /// Returns `None` if version is `0`.
    #[inline]
    pub const fn new(version: u32) -> Option<Self> {
        match std::num::NonZeroU32::new(version) {
            Some(ok) => Some(Self(ok)),
            None => None,
        }
    }

    /// Returns `Version` as `u32`.
    #[inline]
    pub const fn to_u32(self) -> u32 {
        self.0.get()
    }
}

impl std::cmp::PartialEq<u32> for Version {
    #[inline]
    fn eq(&self, other: &u32) -> bool {
        self.0.get().eq(other)
    }
}

impl std::cmp::PartialOrd<u32> for Version {
    #[inline]
    fn partial_cmp(&self, other: &u32) -> Option<std::cmp::Ordering> {
        self.0.get().partial_cmp(other)
    }
}

impl std::fmt::Display for Version {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
