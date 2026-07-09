use std::fmt;

use crate::wayland::error::WlError;

// ===== UnknownId =====

#[derive(Debug, Clone, Copy)]
pub struct UnknownId;

impl WlError for UnknownId {
    #[inline]
    fn code(&self) -> u32 {
        // wl_display::invalid_object
        0
    }

    #[inline]
    fn message(&self) -> &str {
        "unknown object id"
    }
}

impl fmt::Display for UnknownId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message().fmt(f)
    }
}

// ===== OccupiedNewId =====

#[derive(Debug, Clone, Copy)]
pub struct OccupiedNewId;

impl WlError for OccupiedNewId {
    #[inline]
    fn code(&self) -> u32 {
        // wl_display::invalid_method
        1
    }

    #[inline]
    fn message(&self) -> &str {
        "occupied new id"
    }
}

impl fmt::Display for OccupiedNewId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message().fmt(f)
    }
}
