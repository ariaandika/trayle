//! Bitflag operation.

/// Bitflag operation helper.
pub trait Flags: Sized + Copy {
    fn bits(self) -> u32;

    #[inline]
    fn contains(self, other: Self) -> bool {
        self.bits() & other.bits() == other.bits()
    }
}
