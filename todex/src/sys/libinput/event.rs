use std::ffi::c_void;
use std::mem;
use std::ptr::NonNull;

use crate::sys::libinput::context::{Libinput, InputPtr};
use crate::sys::libinput::device::{Device, DevicePtr, DeviceRef};

// ===== Event =====

/// Libinput base event type.
#[repr(transparent)]
pub struct Event(EventPtr);

impl Drop for Event {
    #[inline]
    fn drop(&mut self) {
        unsafe { libinput_event_destroy(self.0) };
    }
}

impl Event {
    /// Forwards to `libinput_get_event`.
    pub(crate) fn get_event(context: &mut Libinput) -> Option<Self> {
        unsafe { libinput_get_event(context.as_ptr()) }
    }

    /// Forwards to `libinput_next_event_type`.
    pub(crate) fn peek_event_kind(context: &Libinput) -> Option<EventKind> {
        let ty = unsafe { libinput_next_event_type(context.as_ptr()) };
        if ty != 0 {
            Some(unsafe { mem::transmute::<i32, EventKind>(ty) })
        } else {
            None
        }
    }

    pub(crate) fn from_raw(ptr: EventPtr) -> Self {
        Self(ptr)
    }

    pub(crate) fn into_raw(self) -> EventPtr {
        mem::ManuallyDrop::new(self).0
    }

    /// Returns the type of the event.
    #[inline]
    pub fn event_kind(&self) -> EventKind {
        unsafe { libinput_event_get_type(self.0) }
    }

    /// Try convert base to specialized event type.
    ///
    /// On success, returns `Ok` with the specialized event type, otherwise this event is returned
    /// back as `Err`.
    #[inline]
    pub fn try_into_type<E: EventType>(self) -> Result<E, Event> {
        E::try_from_event(self)
    }

    /// Return the device associated with this event.
    ///
    /// For device added/removed events this is the device added or removed. For all other device
    /// events, this is the device that generated the event.
    #[inline]
    pub fn device_ref(&self) -> DeviceRef<'_> {
        DeviceRef::new(unsafe { libinput_event_get_device(self.0) })
    }

    /// Return the device associated with this event.
    ///
    /// For device added/removed events this is the device added or removed. For all other device
    /// events, this is the device that generated the event.
    #[inline]
    pub fn device(&self) -> Device {
        self.device_ref().into_owned()
    }
}

impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Event").finish_non_exhaustive()
    }
}

// ===== EventType =====

/// Event type.
pub trait EventType: Sized {
    fn try_from_event(event: Event) -> Result<Self, Event>;
}

// ===== EventKind =====

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum EventKind {
    // /// This is not a real event type, and is only used to tell the user that
    // /// no new event is available in the queue
    // None = 0,

    /// Signals that a device has been added to the context.
    ///
    /// The device will not be read until the next time the user calls [`Input::dispatch`] and data
    /// is available.
    ///
    /// This allows setting up initial device configuration before any events are created.
    DeviceAdded = 1,
    /// Signals that a device has been removed.
    ///
    /// No more events from the associated device will be in the queue or be queued after this
    /// event.
    DeviceRemoved,
    KeyboardKey = 300,
    PointerMotion = 400,
    PointerMotionAbsolute,
    PointerButton,
    /// This event is deprecated as of libinput 1.19.
    ///
    /// Use [`EventKind::PointerScrollWheel`], [`EventKind::PointerScrollFinger`], or
    /// [`EventKind::PointerScrollContinuous`], instead.
    PointerAxis,
    PointerScrollWheel,
    PointerScrollFinger,
    PointerScrollContinuous,
    TouchDown = 500,
    TouchUp,
    TouchMotion,
    TouchCancel,
    TouchFrame,
    TabletToolAxis = 600,
    TabletToolProximity,
    TabletToolTip,
    TabletToolButton,
    TabletPadButton = 700,
    TabletPadRing,
    TabletPadStrip,
    TabletPadKey,
    TabletPadDial,
    GestureSwipeBegin = 800,
    GestureSwipeUpdate,
    GestureSwipeEnd,
    GesturePinchBegin,
    GesturePinchUpdate,
    GesturePinchEnd,
    GestureHoldBegin,
    GestureHoldEnd,
    SwitchToggle = 900,
}

impl EventKind {
    /// Returns `true` if the event type is [`EventKind::DeviceAdded`].
    #[inline]
    pub fn is_device_added(&self) -> bool {
        matches!(self, Self::DeviceAdded)
    }
}

// ===== ffi =====

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct EventPtr(NonNull<c_void>);

unsafe extern "C" {
    fn libinput_get_event(libinput: InputPtr) -> Option<Event>;
    fn libinput_next_event_type(libinput: InputPtr) -> i32;

    fn libinput_event_destroy(event: EventPtr);
    // fn libinput_event_get_context(event: NonNull<libinput_event>) -> *mut c_void;
    fn libinput_event_get_type(event: EventPtr) -> EventKind;
    fn libinput_event_get_device(event: EventPtr) -> DevicePtr;
}
