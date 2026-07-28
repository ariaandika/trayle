//! Bitflag operation.
use std::ops::{BitAnd, BitOr, BitXor};

/// Bitflag operation helper.
pub trait Bitflags<Rhs = Self>: Sized + Copy
where
    Rhs: Bitflags<Bits = Self::Bits>,
{
    type Bits: Bits;

    fn bits(self) -> Self::Bits;

    fn from_bits(bits: Self::Bits) -> Self;

    #[inline]
    fn contains(self, other: Rhs) -> bool {
        self.bits() & other.bits() == other.bits()
    }

    #[inline]
    fn add(self, other: Rhs) -> Self {
        Self::from_bits(self.bits() | other.bits())
    }
}

pub trait Bits:
    Sized + Copy + BitOr<Output = Self> + BitAnd<Output = Self> + BitXor<Output = Self> + PartialEq + Eq
{
}

impl Bits for u32 {}
impl Bits for i32 {}
