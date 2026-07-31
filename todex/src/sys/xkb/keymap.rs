use std::ffi::{CStr, c_char, c_void};
use std::ptr::NonNull;
use std::{marker, slice};

use crate::alloc;
use crate::bitflags::simple_bitflags;
use crate::sys::error::{ErrCode, OsError, simple_os_error};
use crate::sys::macros::simple_ffi;
use crate::sys::xkb::Xkb;

// ===== Keymap =====

/// Compiled keymap object.
///
/// A keymap is immutable after it is created (besides reference counts, etc.).
#[repr(transparent)]
pub struct Keymap(KeymapPtr);

simple_ffi!(impl Drop for Keymap::xkb_keymap_unref);
simple_ffi!(impl Clone for Keymap::xkb_keymap_ref);
simple_ffi!(impl Debug for Keymap);

impl Keymap {
    /// Create a keymap from RMLVO names.
    ///
    /// Caller should prefer passing `None` instead of choosing its own defaults.
    #[inline]
    pub fn new_from_names(
        context: &Xkb,
        names: Option<&RuleNames>,
        format: KeymapFormat,
        flags: CompileFlags,
    ) -> Result<Keymap, KeymapError> {
        unsafe { xkb_keymap_new_from_names2(context.as_ptr(), names, format, flags) }
            .ok_or_else(<_>::errno)
    }

    /// Create a keymap from a keymap string.
    #[inline]
    pub fn new_from_cstr(
        context: &Xkb,
        string: &CStr,
        format: KeymapFormat,
        flags: CompileFlags,
    ) -> Result<Keymap, KeymapError> {
        unsafe { xkb_keymap_new_from_string(context.as_ptr(), string.as_ptr(), format, flags) }
            .ok_or_else(<_>::errno)
    }

    /// Create a keymap from a memory buffer.
    ///
    /// The buffer does not need to be null terminated.
    #[inline]
    pub fn new_from_buffer(
        context: &Xkb,
        buf: &[u8],
        format: KeymapFormat,
        flags: CompileFlags,
    ) -> Result<Keymap, KeymapError> {
        unsafe {
            xkb_keymap_new_from_buffer(
                context.as_ptr(),
                buf.as_ptr().cast(),
                buf.len(),
                format,
                flags,
            )
        }
        .ok_or_else(<_>::errno)
    }

    pub(crate) fn as_ptr(&self) -> KeymapPtr {
        self.0
    }
}

impl Keymap {
    /// Get the compiled keymap as a string.
    #[inline]
    pub fn to_string(
        &self,
        format: KeymapFormat,
        flags: SerializeFlags,
    ) -> Result<KeymapString, SerializeError> {
        // The returned string is dynamically allocated and should be freed by the caller.
        unsafe { xkb_keymap_get_as_string2(self.0, format, flags) }.map_or_else(
            || Err(<_>::errno()),
            |ptr| unsafe {
                Ok(KeymapString {
                    ptr: ptr.cast(),
                    len: CStr::from_ptr(ptr.as_ptr()).count_bytes() + 1,
                })
            },
        )
    }
}

// ===== RuleNames =====

/// Names to compile a keymap with, also known as [RMLVO].
///
/// The names are the common configuration values by which a user picks a keymap.
///
/// [RMLVO]: https://xkbcommon.org/doc/current/xkb-intro.html#RMLVO-intro
#[repr(C)]
pub struct RuleNames<'a> {
    rules: *const c_char,
    model: *const c_char,
    layout: *const c_char,
    variant: *const c_char,
    options: *const c_char,
    _p: marker::PhantomData<&'a CStr>,
}

impl<'a> RuleNames<'a> {
    /// Create new [`RuleNames`].
    #[inline]
    pub fn new(
        rules: &'a CStr,
        model: &'a CStr,
        layout: &'a CStr,
        variant: &'a CStr,
        options: &'a CStr,
    ) -> Self {
        Self {
            rules: rules.as_ptr(),
            model: model.as_ptr(),
            layout: layout.as_ptr(),
            variant: variant.as_ptr(),
            options: options.as_ptr(),
            _p: marker::PhantomData,
        }
    }
}

impl<'a> std::fmt::Debug for RuleNames<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("RuleNames")
                .field("rules", &CStr::from_ptr(self.rules))
                .field("model", &CStr::from_ptr(self.model))
                .field("layout", &CStr::from_ptr(self.layout))
                .field("variant", &CStr::from_ptr(self.variant))
                .field("options", &CStr::from_ptr(self.options))
                .finish()
        }
    }
}

/// The serialized keymap.
///
/// The string is allocated by xkb library, thus [`CString`] cannot be used directly because the
/// restriction that it cannot take ownership from foreign code.
pub struct KeymapString {
    ptr: NonNull<u8>,
    len: usize,
}

impl Drop for KeymapString {
    #[inline]
    fn drop(&mut self) {
        alloc::deallocate(self.ptr);
    }
}

impl KeymapString {
    /// Returns the keymap string as [`CStr`].
    #[inline]
    pub fn as_cstr(&self) -> &CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(self.as_bytes()) }
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl std::ops::Deref for KeymapString {
    type Target = CStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_cstr()
    }
}

impl std::fmt::Debug for KeymapString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_cstr().fmt(f)
    }
}

// ===== KeymapFormat =====

/// The possible keymap formats.
///
/// See [xkbcommon docs][xd] for what keymap format to use.
///
/// [xd]: <https://xkbcommon.org/doc/current/group__keymap.html#gab0f75d6cc5773e5dd404e2c3f61366a3>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum KeymapFormat {
    /// Use the format from which the keymap was originally created.
    UseOriginalFormat = -1,
    /// The classic XKB text format, as generated by `xkbcomp -xkb`.
    TextV1 = 1,
    /// Xkbcommon extensions of the classic XKB text format, incompatible with X11.
    TextV2 = 2,
}

// ===== CompileFlags =====

/// Flags for keymap compilation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct CompileFlags(u32);

simple_bitflags!(CompileFlags);

impl CompileFlags {
    /// Do not apply any flags.
    pub const NO_FLAGS: Self = Self(0);
}

// ===== SerializeFlags =====

/// Flags to control keymap serialization.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct SerializeFlags(u32);

simple_bitflags!(SerializeFlags);

impl SerializeFlags {
    /// Do not apply any flags.
    pub const NO_FLAGS: Self = Self(0);
    /// Enable pretty-printing.
    pub const PRETTY: Self = Self(1 << 0);
    /// Do not drop unused bits (key types, compatibility entries).
    pub const KEEP_UNUSED: Self = Self(1 << 1);
}

// ===== error =====

/// An error that can occur during keymap creation.
#[derive(Clone, Copy)]
pub struct KeymapError(ErrCode);

simple_os_error!(KeymapError, "create xkb keymap context");

/// An error that can occur during keymap serialization.
#[derive(Clone, Copy)]
pub struct SerializeError(ErrCode);

simple_os_error!(SerializeError, "serialize xkb keymap");

// ===== ffi =====

type ContextPtr = NonNull<c_void>;
type KeymapPtr = NonNull<c_void>;

unsafe extern "C" {
    fn xkb_keymap_new_from_names2(
        context: ContextPtr,
        names: Option<&RuleNames>,
        format: KeymapFormat,
        flags: CompileFlags,
    ) -> Option<Keymap>;
    fn xkb_keymap_new_from_string(
        context: ContextPtr,
        string: *const c_char,
        format: KeymapFormat,
        flags: CompileFlags,
    ) -> Option<Keymap>;
    fn xkb_keymap_new_from_buffer(
        context: ContextPtr,
        string: *const c_char,
        length: usize,
        format: KeymapFormat,
        flags: CompileFlags,
    ) -> Option<Keymap>;
    fn xkb_keymap_ref(keymap: KeymapPtr) -> KeymapPtr;
    fn xkb_keymap_unref(keymap: KeymapPtr);
    fn xkb_keymap_get_as_string2(
        keymap: KeymapPtr,
        format: KeymapFormat,
        flags: SerializeFlags,
    ) -> Option<NonNull<c_char>>;
}
