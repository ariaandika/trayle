pub use crate::wayland::interface::wl_display::DisplayError;
pub use crate::wayland::object::ObjectError;
pub use crate::wayland::wire::DecodeError;

pub trait WlError {
    fn code(&self) -> u32;

    fn message(&self) -> &str;
}

impl WlError for ObjectError {
    fn code(&self) -> u32 {
        DisplayError::InvalidObject as u32
    }

    fn message(&self) -> &str {
        self.message()
    }
}

impl WlError for DecodeError {
    fn code(&self) -> u32 {
        DisplayError::InvalidMethod as u32
    }

    fn message(&self) -> &str {
        self.message()
    }
}

macro_rules! delegate_protocol_error {
    ($wl_ty:ident) => {
        impl WlError for crate::wayland::interface::$wl_ty::Error {
            fn code(&self) -> u32 {
                *self as u32
            }

            fn message(&self) -> &str {
                self.message()
            }
        }
    };
}
delegate_protocol_error!(wl_surface);
delegate_protocol_error!(wl_shm);
delegate_protocol_error!(wl_seat);
