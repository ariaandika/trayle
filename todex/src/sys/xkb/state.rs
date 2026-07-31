use std::ffi::{c_int, c_void};
use std::ptr::{self, NonNull};
use std::slice;

use crate::bitflags::simple_bitflags;
use crate::sys::error::{ErrCode, OsError, simple_os_error};
use crate::sys::macros::simple_ffi;
use crate::sys::xkb::{KeySym, Keymap};

// ===== Keymap =====

/// Keyboard state object.
#[repr(transparent)]
pub struct KeyboardState(StatePtr);

simple_ffi!(impl Drop for KeyboardState::xkb_state_unref);
simple_ffi!(impl Clone for KeyboardState::xkb_state_ref);
simple_ffi!(impl Debug for KeyboardState);

impl KeyboardState {
    /// Create new keyboard state.
    #[inline]
    pub fn new(keymap: &Keymap) -> Result<KeyboardState, StateError> {
        unsafe { xkb_state_new(keymap.as_ptr()) }.ok_or_else(<_>::errno)
    }

    /// Update the keyboard state to reflect a given key being pressed or released.
    ///
    /// This entry point is intended for server applications and should not be used by client
    /// applications.
    ///
    /// Note that XKB keycode are different with linux keycode.
    ///
    /// To convert it: `xkb_keycode = linux_keycode + 8`
    #[inline]
    pub fn update_key<K: Into<KeyCode>>(&mut self, keycode: K, dir: KeyDirection) -> Component {
        unsafe { xkb_state_update_key(self.0, keycode.into(), dir) }
    }

    /// Get the keysyms obtained from pressing a particular key in a given keyboard state.
    ///
    /// If no keysyms are produced by the key in the given keyboard state, returns empty slice.
    ///
    /// This method accept linux keycode, which will be automatically converted into xkb keycode.
    ///
    /// As an extension to XKB, this function can return more than one keysym. If you do not want to
    /// handle this case, you can use [`KeyboardState::keysym`] for a simpler interface.
    ///
    /// This function performs Capitalization Keysym Transformations.
    #[inline]
    pub fn keysyms<K: Into<KeyCode>>(&self, keycode: K) -> &[KeySym] {
        unsafe {
            let mut out = ptr::dangling();
            let len = xkb_state_key_get_syms(self.0, keycode.into(), &mut out);
            // `out` set to `NULL` if no keysyms are produced
            slice::from_raw_parts(
                if out.is_null() { ptr::dangling() } else { out },
                len as usize,
            )
        }
    }

    /// Get the single keysym obtained from pressing a particular key in a given keyboard state.
    ///
    /// If the key does not have exactly one keysym, returns `None`.
    ///
    /// This method accept linux keycode, which will be automatically converted into xkb keycode.
    ///
    /// This function is similar [`KeyboardState::keysyms`], but intended for callers which cannot
    /// or do not want to handle the case where multiple keysyms are returned (in which case this
    /// function is preferred).
    #[inline]
    pub fn keysym<K: Into<KeyCode>>(&self, keycode: K) -> Option<KeySym> {
        unsafe { xkb_state_key_get_one_sym(self.0, keycode.into()) }
    }

    /// A a mask representing the given components of the modifier state.
    #[inline]
    pub fn serialize_mods(&self, components: Component) -> u32 {
        unsafe { xkb_state_serialize_mods(self.0, components) }
    }
}

// ===== KeyCode =====

/// XKB keycode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct KeyCode(u32);

impl KeyCode {
    /// Create XKB keycode from linux keycode.
    #[inline]
    pub fn from_linux_keycode(keycode: u32) -> Self {
        Self(keycode + 8)
    }
}

// ===== KeyDirection =====

/// Specifies the direction of the key (press / release).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum KeyDirection {
    /// The key was released.
    Up,
    /// The key was pressed.
    Down,
    /// The key was repeated.
    Repeated,
}

// ===== StateComponent =====

/// Modifier and layout types for state objects.
///
/// In XKB, the DEPRESSED components are also known as base.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Component(u32);

simple_bitflags!(Component);

impl std::fmt::Binary for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Component").field(&format_args!("{:b}", self.0)).finish()
    }
}

impl Component {
    /// Depressed modifiers, i.e. a key is physically holding them.
    pub const MODS_DEPRESSED: Self = Self(1 << 0);
    /// Latched modifiers, i.e. will be unset after the next non-modifier key press.
    pub const MODS_LATCHED: Self = Self(1 << 1);
    /// Locked modifiers, i.e. will be unset after the key provoking the lock has been pressed
    /// again.
    pub const MODS_LOCKED: Self = Self(1 << 2);
    /// Effective modifiers, i.e. currently active and affect key processing (derived from the other
    /// state components).
    ///
    /// Use this unless you explicitly care how the state came about.
    pub const MODS_EFFECTIVE: Self = Self(1 << 3);
    /// Depressed layout, i.e. a key is physically holding it.
    pub const LAYOUT_DEPRESSED: Self = Self(1 << 4);
    /// Latched layout, i.e. will be unset after the next non-modifier key press.
    pub const LAYOUT_LATCHED: Self = Self(1 << 5);
    /// Locked layout, i.e. will be unset after the key provoking the lock has been pressed again.
    pub const LAYOUT_LOCKED: Self = Self(1 << 6);
    /// Effective layout, i.e. currently active and affects key processing (derived from the other
    /// state components).
    ///
    /// Use this unless you explicitly care how the state came about.
    pub const LAYOUT_EFFECTIVE: Self = Self(1 << 7);
    /// LEDs (derived from the other state components).
    pub const LEDS: Self = Self(1 << 8);
    /// Effective keyboard controls.
    pub const CONTROLS: Self = Self(1 << 9);
}

// ===== error =====

/// An error that can occur during xkb keyboard state creation.
#[derive(Clone, Copy)]
pub struct StateError(ErrCode);

simple_os_error!(StateError, "create xkb keyboard state");

// ===== ffi =====

type KeymapPtr = NonNull<c_void>;
type StatePtr = NonNull<c_void>;

unsafe extern "C" {
    fn xkb_state_new(keymap: KeymapPtr) -> Option<KeyboardState>;
    fn xkb_state_ref(state: StatePtr) -> StatePtr;
    fn xkb_state_unref(state: StatePtr);
    fn xkb_state_update_key(state: StatePtr, keycode: KeyCode, dir: KeyDirection) -> Component;
    fn xkb_state_key_get_syms(state: StatePtr, keycode: KeyCode, syms_out: *mut *const KeySym) -> c_int;
    fn xkb_state_key_get_one_sym(state: StatePtr, keycode: KeyCode) -> Option<KeySym>;
    fn xkb_state_serialize_mods(state: StatePtr, components: Component) -> u32;
}
