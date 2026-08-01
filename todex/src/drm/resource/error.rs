use crate::drm::ioctl::{ErrCode, simple_os_error};

/// An error that can occur during resource request.
#[derive(Clone, Copy)]
pub struct ResourceError(ErrCode);

simple_os_error!(ResourceError, "request drm resource");
