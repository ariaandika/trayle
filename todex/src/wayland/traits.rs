use crate::wayland::{AsObjectId, FromObjectId, Interface};

// ===== interface =====

/// Type that is associated with an interface.
pub trait AsInterface {
    /// Returns the interface that this type associated with.
    fn interface(&self) -> Interface;
}

// ===== opcode =====

/// Request/event opcode.
///
/// This type is the exhaustive list of the valid opcodes.
pub trait OpCode: Sized {
    /// Creates this type from raw opcode.
    ///
    /// Returns `None` if raw value is invalid for this type.
    fn from_op(op: u16) -> Option<Self>;

    /// Converts to raw opcode.
    fn to_op(self) -> u16;
}

/// Type that is associated with an opcode.
pub trait AsOpCode {
    /// The opcode type.
    type OpCode: OpCode;

    /// The opcode value.
    const OPCODE: Self::OpCode;

    /// The opcode wayland name.
    const OPNAME: &str;
}

// ===== enum =====

/// Type that represent a wayland enum.
pub trait WlEnum: Sized {
    /// Create enum from integer.
    ///
    /// Returns `None` if the integer did not represent valid entry.
    fn from_u32(uint: u32) -> Option<Self>;

    /// Returns `u32` representation of the enum.
    fn to_u32(self) -> u32;
}

// ===== object =====

/// Type that represent a wayland object.
pub trait WlObject: FromObjectId + AsObjectId + AsInterface {}

impl<O: FromObjectId + AsObjectId + AsInterface> WlObject for O {}

// ===== operation =====

pub trait Operation: AsInterface + AsOpCode {
    const IS_REQUEST: bool;

    const IS_EVENT: bool = !Self::IS_REQUEST;

    const IS_DESTRUCTOR: bool = false;
}
