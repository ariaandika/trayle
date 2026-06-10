use crate::wayland::{AsObjectId, FromObjectId, Interface, NewId, WlError};

// ===== interface =====

/// Type that is associated with an interface.
pub trait AsInterface {
    /// The interface value.
    const INTERFACE: Interface;

    /// Wayland name of this interface.
    const INTERFACE_NAME: &str;
}

impl<T: AsInterface> AsInterface for NewId<T> {
    const INTERFACE: Interface = T::INTERFACE;

    const INTERFACE_NAME: &str = T::INTERFACE_NAME;
}

// ===== opcode =====

/// Request/event opcode.
///
/// This type is the exhaustive list of the valid opcodes.
pub trait OpCode: Sized {
    /// Wayland name of this opcode.
    const OPNAME: &str;

    /// Creates this type from raw opcode.
    ///
    /// Returns `None` if raw value is invalid for this type.
    fn from_op(op: u16) -> Option<Self>;

    /// Converts to raw opcode.
    fn to_op(self) -> u16;

    /// Creates this type from raw opcode.
    ///
    /// Returns `Err` if raw value is invalid for this type.
    #[inline]
    fn try_from_op(op: u16) -> Result<Self, WlError> {
        Self::from_op(op).ok_or(WlError::UnknownOp)
    }
}

/// Type that is associated with an opcode.
pub trait AsOpCode {
    /// The opcode type.
    type OpCode: OpCode;

    /// The opcode value.
    const OPCODE: Self::OpCode;
}

// ===== object =====

/// Type that represent a wayland object.
pub trait WlObject: FromObjectId + AsObjectId + AsInterface {}

impl<O: FromObjectId + AsObjectId + AsInterface> WlObject for O {}

// ===== enum =====

/// Type that represent a wayland enum.
pub trait WlEnum: Sized {
    /// Create enum from integer.
    ///
    /// Returns `None` if the integer did not represent valid entry.
    fn from_u32(uint: u32) -> Option<Self>;

    /// Returns `u32` representation of the enum.
    fn to_u32(self) -> u32;

    /// Create enum from integer.
    ///
    /// Returns `Err` if the integer did not represent valid entry.
    #[inline]
    fn try_from_u32(uint: u32) -> Result<Self, WlError> {
        Self::from_u32(uint).ok_or(WlError::UnknownEnumEntry)
    }
}

