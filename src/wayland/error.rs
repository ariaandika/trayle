use crate::wayland::id::ZeroId;

#[derive(Debug)]
pub enum WlError {
    /// Unknown op code.
    UnknownOp,
    /// Unknown object id.
    UnknownObject,
    /// Invalid size for message payload.
    InvalidSize,
    /// Invalid object id of `0`.
    ZeroId,
}

impl std::error::Error for WlError { }

impl std::fmt::Display for WlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownOp => write!(f, "unknown op code"),
            Self::UnknownObject => write!(f, "unknown object id"),
            Self::InvalidSize => write!(f, "invalid payload size"),
            Self::ZeroId => write!(f, "invalid object id of `0`"),
        }
    }
}

impl From<ZeroId> for WlError {
    fn from(_: ZeroId) -> Self {
        Self::ZeroId
    }
}
