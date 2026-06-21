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
