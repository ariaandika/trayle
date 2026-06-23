use crate::wayland::wire::DecodeError;

/// Type that can be represented as raw opcode.
pub trait OpCode: Sized {
    /// Creates this type from raw opcode.
    ///
    /// Returns `None` if the value is invalid for this type.
    fn from_op(op: u16) -> Option<Self>;

    /// Converts to raw opcode.
    fn to_op(self) -> u16;

    /// Creates this type from raw opcode.
    ///
    /// Returns `Err` if the value is invalid for this type.
    #[inline]
    fn try_from_op(op: u16) -> Result<Self, DecodeError> {
        Self::from_op(op).ok_or(DecodeError::UnknownOpCode)
    }
}

/// Type that is associated with an opcode.
pub trait AsOpCode {
    /// The opcode type.
    type OpCode: OpCode;

    /// The opcode value.
    const OPCODE: Self::OpCode;

    /// The operation wayland name.
    const OPNAME: &str;
}
