use std::ffi::c_void;
use std::ptr::NonNull;

use crate::bitflags::simple_bitflags;
use crate::sys::error::{ErrCode, OsError, simple_os_error};
use crate::sys::macros::simple_ffi;

// ===== Xkb =====

/// Xkb context.
#[repr(transparent)]
pub struct Xkb(ContextPtr);

simple_ffi!(impl Drop for Xkb::xkb_context_unref);
simple_ffi!(impl Clone for Xkb::xkb_context_ref);
simple_ffi!(impl Debug for Xkb);

impl Xkb {
    /// Create new [`Xkb`] context.
    #[inline]
    pub fn new(flags: ContextFlags) -> Result<Xkb, ContextError> {
        unsafe { xkb_context_new(flags) }.ok_or_else(<_>::errno)
    }

    pub(crate) fn as_ptr(&self) -> ContextPtr {
        self.0
    }
}

// ===== ContextFlags =====

/// Flags for context creation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ContextFlags(u32);

simple_bitflags!(ContextFlags);

impl ContextFlags {
    /// Do not apply any context flags.
    pub const NO_FLAGS: Self = Self(0);
    /// Create this context with an empty include path.
    pub const NO_DEFAULT_INCLUDES: Self = Self(1 << 0);
    /// Don’t take RMLVO names from the environment.
    pub const NO_ENVIRONMENT_NAMES: Self = Self(1 << 1);
    /// Disable the use of secure_getenv for this context, so that privileged processes can use
    /// environment variables.
    pub const NO_SECURE_GETENV: Self = Self(1 << 2);
}

// ===== error =====

/// An error that can occur during xkb context creation.
#[derive(Clone, Copy)]
pub struct ContextError(ErrCode);

simple_os_error!(ContextError, "create xkb context");

// ===== ffi =====

type ContextPtr = NonNull<c_void>;

unsafe extern "C" {
    fn xkb_context_new(flags: ContextFlags) -> Option<Xkb>;
    fn xkb_context_ref(context: ContextPtr) -> ContextPtr;
    fn xkb_context_unref(context: ContextPtr);
}
