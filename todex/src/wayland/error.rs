use crate::wayland::wl_registry::BindError;
use crate::wayland::{DecodeError, MessageError, ObjectError};

#[derive(Debug, Clone, Copy)]
pub enum WlError {
    /// Not yet implemented.
    NotYetImplemented,
    /// Message error.
    Message(MessageError),
    /// Decode error.
    Decode(DecodeError),
    /// Object error.
    Object(ObjectError),
    /// Registry bind error.
    Bind(BindError),
}

impl WlError {
    #[inline]
    pub fn message(&self) -> &'static str {
        match self {
            Self::NotYetImplemented => "not yet implemented",
            Self::Message(e) => e.message(),
            Self::Decode(e) => e.message(),
            Self::Object(e) => e.message(),
            Self::Bind(e) => e.message(),
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

impl From<DecodeError> for WlError {
    #[inline]
    fn from(v: DecodeError) -> Self {
        Self::Decode(v)
    }
}

impl From<ObjectError> for WlError {
    #[inline]
    fn from(v: ObjectError) -> Self {
        Self::Object(v)
    }
}
impl From<BindError> for WlError {
    #[inline]
    fn from(v: BindError) -> Self {
        Self::Bind(v)
    }
}

