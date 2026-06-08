#[derive(Debug, Clone, Copy)]
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
