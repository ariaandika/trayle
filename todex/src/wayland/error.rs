use crate::wayland::object::ObjectError;
use crate::wayland::wire::DecodeError;
use crate::wayland::interface::wl_display::DisplayError as WlDisplayError;
use crate::wayland::interface::wl_seat;
use crate::wayland::interface::wl_shm::Error as ShmError;

// ===== BindError =====

#[derive(Debug, Clone, Copy)]
pub enum BindError {
    /// Unknown bind name.
    UnknownName,
    /// Missmatch bind name.
    MissmatchName,
    /// Unsupported bind version.
    UnsupportedVersion,
}

impl BindError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::UnknownName => "unknown bind name",
            Self::MissmatchName => "missmatch bind name",
            Self::UnsupportedVersion => "unsupported bind version"
        }
    }
}

// ===== WlError =====

#[derive(Debug, Clone, Copy)]
pub enum WlError {
    /// Not yet implemented.
    NotYetImplemented,
    /// Seat error.
    Seat(wl_seat::Error),
    /// Decode error.
    Decode(DecodeError),
    /// Object error.
    Object(ObjectError),
    /// Registry bind error.
    Bind(BindError),
    /// Shm operation error.
    Shm(ShmError),
}

const SEMANTIC: u32 = WlDisplayError::InvalidObject as u32;
const MALFORMED: u32 = WlDisplayError::InvalidMethod as u32;
const IMPLEMENTATION: u32 = WlDisplayError::Implementation as u32;

impl WlError {
    #[inline]
    pub fn message(&self) -> &'static str {
        match self {
            Self::NotYetImplemented => "not yet implemented",
            Self::Seat(e) => match e {
                wl_seat::Error::MissingCapability => "seat missing capability",
            }
            Self::Decode(e) => e.message(),
            Self::Object(e) => e.message(),
            Self::Bind(e) => e.message(),
            Self::Shm(e) => e.message(),
        }
    }

    #[inline]
    pub fn code(&self) -> u32 {
        match self {
            WlError::NotYetImplemented => IMPLEMENTATION,
            WlError::Seat(_) => IMPLEMENTATION,
            WlError::Decode(_) => MALFORMED,
            WlError::Object(_) => SEMANTIC,
            WlError::Bind(_) => MALFORMED,
            WlError::Shm(_) => SEMANTIC,
        }
    }
}

impl std::error::Error for WlError { }

impl std::fmt::Display for WlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

macro_rules! from {
    ($($v:ident($e:ty)),* $(,)?) => {
        $(
            impl From<$e> for WlError {
                #[inline]
                fn from(v: $e) -> Self {
                    Self::$v(v)
                }
            }
        )*
    };
}
from! {
    Seat(wl_seat::Error),
    Decode(DecodeError),
    Object(ObjectError),
    Bind(BindError),
    Shm(ShmError),
}
