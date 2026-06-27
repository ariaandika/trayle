// ===== Enum =====

/// Type that represent a wayland enum.
///
/// # Protocol Violation
///
/// Note that so far, there is no practical difference between using `int` and `uint` to represent
/// an `enum`. Therefore, any args with an enum of `int`, will be casted to `u32` before transformed
/// into the enum type, and vice versa.
pub trait WlEnum: Sized {
    /// Create enum from `uint`.
    ///
    /// Returns `None` if the integer did not represent valid entry.
    fn from_u32(uint: u32) -> Option<Self>;

    /// Returns `uint` representation of the enum.
    fn to_u32(self) -> u32;

    /// Create enum from `int`.
    ///
    /// The default implementation is to passed the int to [`WlEnum::from_u32`]. See the trait docs
    /// for more detail.
    #[inline]
    fn from_i32(int: i32) -> Option<Self> {
        Self::from_u32(int as u32)
    }

    /// Create enum from `int`.
    ///
    /// The default implementation is to cast the result of [`WlEnum::to_u32`]. See the trait docs
    /// for more detail.
    #[inline]
    fn to_i32(self) -> i32 {
        self.to_u32() as i32
    }
}
