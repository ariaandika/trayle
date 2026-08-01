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

macro_rules! simple_bitflags {
    ($me:ty, $bits:ty) => {
        impl crate::bitflags::Bitflags for $me {
            type Bits = $bits;

            #[inline]
            fn bits(self) -> Self::Bits {
                self.0
            }

            #[inline]
            fn from_bits(bits: Self::Bits) -> Self {
                Self(bits)
            }
        }
    };
    ($me:ty) => {
        crate::bitflags::simple_bitflags!($me, u32);
    }
}
pub(crate) use simple_bitflags;
macro_rules! simple_bitflags_debug {
    ($me:ident, $($entries:ident),*) => {
        impl fmt::Debug for $me {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let entries = [$((Self::$entries,stringify!($entries)),)*];
                let mut has_flag = false;
                write!(f, concat!(stringify!($me), "("))?;
                for (mode, name) in entries {
                    use crate::bitflags::Bitflags;
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
    };
}
pub(crate) use simple_bitflags_debug;
