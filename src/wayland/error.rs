use crate::wayland::id::ZeroId;

#[derive(Debug, Clone, Copy)]
pub enum WlError {
    /// Unknown op code.
    UnknownOp,
    /// Unknown object id.
    UnknownObject,
    /// Unknown global when binding in `wl_registry::bind`.
    UnknownBind,
    /// Invalid payload size.
    InvalidSize,
    /// Excessive payload size.
    ExcessiveSize,
    /// Invalid new object id, e.g: new id that is used by existing object.
    InvalidNewId,
    /// Invalid object id of `0`.
    ZeroId,
    /// Invalid null value.
    Null,
    /// Internal compositor error.
    Internal,
}

impl WlError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::UnknownOp => "unknown op code",
            Self::UnknownObject => "unknown object id",
            Self::UnknownBind => "unknown global binding operation",
            Self::InvalidSize => "invalid payload size",
            Self::ExcessiveSize => "excessive payload size",
            Self::InvalidNewId => "invalid client new object id",
            Self::ZeroId => "invalid object id of `0`",
            Self::Null => "invalid null value",
            Self::Internal => "internal error",
        }
    }

    #[allow(unused)]
    pub const fn todo<T>() -> Result<T, WlError> {
        Err(WlError::Internal)
    }
}

impl std::error::Error for WlError { }

impl std::fmt::Display for WlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl From<ZeroId> for WlError {
    fn from(_: ZeroId) -> Self {
        Self::ZeroId
    }
}
