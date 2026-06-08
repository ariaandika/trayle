use crate::wayland::{AsObjectId, FromObjectId, Interface, NewId, WlError};

/// Request/event opcode.
pub trait OpCode: Sized {
    fn from_op(op: u16) -> Option<Self>;

    fn to_op(self) -> u16;

    #[inline]
    fn try_from_op(op: u16) -> Result<Self, WlError> {
        Self::from_op(op).ok_or(WlError::UnknownOp)
    }
}

/// Type that is belong to an interface
pub trait AsInterface {
    /// The interface this object belongs to.
    const INTERFACE: Interface;
}

impl<T: AsInterface> AsInterface for NewId<T> {
    const INTERFACE: Interface = T::INTERFACE;
}

/// Type that represent a wayland object.
pub trait WlObject: FromObjectId + AsObjectId + AsInterface {}

impl<O: FromObjectId + AsObjectId + AsInterface> WlObject for O {}

/// Wayland enum.
pub trait WlEnum: Sized {
    /// Create enum from integer.
    ///
    /// Returns `None` if the integer did not represent any entry.
    fn from_u32(uint: u32) -> Option<Self>;

    /// Returns `u32` representation of the enum.
    fn to_u32(self) -> u32;

    #[inline]
    fn try_from_u32(uint: u32) -> Result<Self, WlError> {
        Self::from_u32(uint).ok_or(WlError::UnknownEnumEntry)
    }
}

// ===== implementations =====

impl<T: AsInterface> AsInterface for crate::wayland::Encodable<T> {
    const INTERFACE: Interface = T::INTERFACE;
}

