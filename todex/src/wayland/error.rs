use crate::wayland::MessageError;

#[derive(Debug, Clone, Copy)]
pub enum WlError {
    /// Unknown op code.
    UnknownOp,
    /// Unknown object id.
    UnknownObject,
    /// Unknown global when binding in `wl_registry::bind`.
    UnknownBind,
    /// Unknown enum variant from given integer.
    UnknownEnumEntry,
    /// Missmatched interface for given object id.
    InvalidObject,
    /// Invalid payload size.
    InvalidSize,
    /// Invalid new object id, e.g: new id that is used by existing object.
    InvalidNewId,
    /// Invalid object id of `0`.
    ZeroId,
    /// Invalid null value.
    Null,
    /// Non-UTF8 string.
    NonUtf8,
    /// No fd in ancillary data.
    MissingFd,
    /// Not yet implemented.
    NotYetImplemented,
    /// Message error.
    Message(MessageError),
}

impl WlError {
    #[inline]
    pub fn message(&self) -> &'static str {
        match self {
            Self::UnknownOp => "unknown op code",
            Self::UnknownObject => "unknown object id",
            Self::UnknownBind => "unknown global binding operation",
            Self::UnknownEnumEntry => "unknown enum variant from given integer",
            Self::InvalidObject => "missmatched interface for given object id",
            Self::InvalidSize => "invalid payload size",
            Self::InvalidNewId => "invalid client new object id",
            Self::ZeroId => "invalid object id of `0`",
            Self::Null => "invalid null value",
            Self::NonUtf8 => "non-utf8 string",
            Self::MissingFd => "no fd in ancillary data",
            Self::NotYetImplemented => "not yet implemented",
            Self::Message(e) => e.message(),
        }
    }
}

impl std::error::Error for WlError { }

impl std::fmt::Display for WlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl From<MessageError> for WlError {
    #[inline]
    fn from(v: MessageError) -> Self {
        Self::Message(v)
    }
}
