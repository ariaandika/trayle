/// An error that can occur during decoding operation.
#[derive(Debug, Clone, Copy)]
pub enum DecodeError {
    /// Insufficient payload size.
    InsufficientSize,
    /// Excessive message size.
    ExcessiveSize,
    /// Invalid object id of `0`.
    ZeroId,
    /// Invalid null value.
    Null,
    /// Non-UTF8 string.
    NonUtf8,
    /// Unknown enum entry.
    UnknownEnumEntry,
    /// Unknown op code.
    UnknownOpCode,
    /// Missing fd.
    MissingFd,
    /// Invalid version.
    InvalidVersion,
}

impl DecodeError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::InsufficientSize => "insufficient payload size",
            Self::ExcessiveSize => "excessize message size",
            Self::ZeroId => "invalid object id of 0",
            Self::Null => "invalid null value",
            Self::NonUtf8 => "non utf-8 string",
            Self::UnknownEnumEntry => "unknown enum entry",
            Self::UnknownOpCode => "unknown op code",
            Self::MissingFd => "missing fd",
            Self::InvalidVersion => "invalid version",
        }
    }
}
