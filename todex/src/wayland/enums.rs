/// Type that represent a wayland enum.
pub trait WlEnum: Sized {
    /// Create enum from integer.
    ///
    /// Returns `None` if the integer did not represent valid entry.
    fn from_u32(uint: u32) -> Option<Self>;

    /// Returns `u32` representation of the enum.
    fn to_u32(self) -> u32;
}
