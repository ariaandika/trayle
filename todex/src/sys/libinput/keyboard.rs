use std::ffi::c_void;
use std::ptr::NonNull;

use crate::sys::libinput::event::{Event, EventKind, EventPtr, EventType};

// ===== Event =====

/// A keyboard event representing a key press/release.
#[repr(transparent)]
pub struct Keyboard(KeyboardPtr);

impl Drop for Keyboard {
    fn drop(&mut self) {
        Event::from_raw(unsafe { libinput_event_keyboard_get_base_event(self.0) });
    }
}

impl EventType for Keyboard {
    #[inline]
    fn try_from_event(event: Event) -> Result<Self, Event> {
        if event.event_kind() == EventKind::KeyboardKey {
            Ok(Self(unsafe {
                libinput_event_get_keyboard_event(event.into_raw())
            }))
        } else {
            Err(event)
        }
    }
}

impl Keyboard {
    /// Returns the event time for this event
    ///
    /// Timestamps may not always increase. See the libinput documentation for more details.
    #[inline]
    pub fn time(&self) -> u32 {
        unsafe { libinput_event_keyboard_get_time(self.0) }
    }

    /// Returns the event time for this event in microseconds
    ///
    /// Timestamps may not always increase. See the libinput documentation for more details.
    #[inline]
    pub fn time_usec(&self) -> u64 {
        unsafe { libinput_event_keyboard_get_time_usec(self.0) }
    }

    /// Returns the keycode that triggered this key event.
    #[inline]
    pub fn key(&self) -> u32 {
        unsafe { libinput_event_keyboard_get_key(self.0) }
    }

    /// Returns the state change of the key.
    #[inline]
    pub fn key_state(&self) -> i32 {
        unsafe { libinput_event_keyboard_get_key_state(self.0) }
    }

    /// Returns the seat wide pressed key count for the key of this event.
    ///
    /// Returns the total number of keys pressed on all devices on the associated seat after the
    /// event was triggered.
    #[inline]
    pub fn seat_key_count(&self) -> u32 {
        unsafe { libinput_event_keyboard_get_seat_key_count(self.0) }
    }
}

impl std::fmt::Debug for Keyboard {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keyboard")
            .field("key", &self.key())
            .field("key_state", &self.key_state())
            .finish()
    }
}

// ===== ffi =====

type KeyboardPtr = NonNull<c_void>;

unsafe extern "C" {
    fn libinput_event_get_keyboard_event(event: EventPtr) -> KeyboardPtr;
    fn libinput_event_keyboard_get_time(kb: KeyboardPtr) -> u32;
    fn libinput_event_keyboard_get_time_usec(kb: KeyboardPtr) -> u64;
    fn libinput_event_keyboard_get_key(kb: KeyboardPtr) -> u32;
    fn libinput_event_keyboard_get_key_state(kb: KeyboardPtr) -> i32;
    fn libinput_event_keyboard_get_base_event(kb: KeyboardPtr) -> EventPtr;
    fn libinput_event_keyboard_get_seat_key_count(kb: KeyboardPtr) -> u32;
}
