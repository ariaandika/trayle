pub use crate::wayland::interface::wl_display::DisplayError;
pub use crate::wayland::object::ObjectError;
pub use crate::wayland::wire::DecodeError;

pub trait WlError {
    fn code(&self) -> u32;

    fn message(&self) -> &str;
}

impl WlError for ObjectError {
    #[inline]
    fn code(&self) -> u32 {
        DisplayError::InvalidObject as u32
    }

    #[inline]
    fn message(&self) -> &str {
        self.message()
    }
}

impl WlError for DecodeError {
    #[inline]
    fn code(&self) -> u32 {
        DisplayError::InvalidMethod as u32
    }

    #[inline]
    fn message(&self) -> &str {
        self.message()
    }
}
