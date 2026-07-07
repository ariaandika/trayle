// ===== Fixed =====

/// Wayland `fixed` primitive types.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

impl std::fmt::Display for Fixed {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_f32().fmt(f)
    }
}

impl std::fmt::Debug for Fixed {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Fixed").field(&self.to_f32()).finish()
    }
}
