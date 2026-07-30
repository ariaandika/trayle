use std::ffi::{c_double, c_int, c_void};
use std::ptr::NonNull;

use crate::sys::libinput::event::{Event, EventKind, EventPtr, EventType};

/// A pointer event representing relative or absolute pointer movement, a button press/release or
/// scroll axis events.
///
/// # Pointer Acceleration
///
/// Pointer acceleration is a function to convert input deltas to output deltas, usually based on
/// the movement speed of the device, see Pointer acceleration for details.
///
/// Pointer acceleration is normalized into a [-1, 1] range, where -1 is “slowest” and 1 is
/// “fastest”. Most devices use a default speed of 0.
///
/// The pointer acceleration profile defines how the input deltas are converted, see Pointer
/// acceleration profiles. Most devices have their default profile (usually called “adaptive”) and a
/// “flat” profile. The flat profile does not apply any acceleration.
#[repr(transparent)]
pub struct Pointer(PointerPtr);

impl Drop for Pointer {
    fn drop(&mut self) {
        Event::from_raw(unsafe { libinput_event_pointer_get_base_event(self.0) });
    }
}

impl EventType for Pointer {
    #[inline]
    fn try_from_event(event: Event) -> Result<Self, Event> {
        use EventKind as E;
        if matches!(
            event.event_kind(),
            E::PointerAxis
                | E::PointerMotion
                | E::PointerButton
                | E::PointerScrollWheel
                | E::PointerScrollFinger
                | E::PointerMotionAbsolute
                | E::PointerScrollContinuous
        ) {
            Ok(Self(unsafe {
                libinput_event_get_pointer_event(event.into_raw())
            }))
        } else {
            Err(event)
        }
    }
}

impl Pointer {
    /// Returns the event time for this event.
    ///
    /// Timestamps may not always increase.
    #[inline]
    pub fn time(&self) -> u32 {
        unsafe { libinput_event_pointer_get_time(self.0) }
    }

    /// Returns the event time for this event in microseconds.
    ///
    /// Timestamps may not always increase.
    #[inline]
    pub fn time_usec(&self) -> u64 {
        unsafe { libinput_event_pointer_get_time_usec(self.0) }
    }

    /// Returns the x delta between the last event and the current event.
    ///
    /// If a device employs pointer acceleration, the delta returned by this function is the
    /// accelerated delta. Relative motion deltas are to be interpreted as pixel movement of a
    /// standardized mouse.
    #[inline]
    pub fn dx(&self) -> f64 {
        // It is an application bug to call this function for events other than
        // LIBINPUT_EVENT_POINTER_MOTION.
        unsafe { libinput_event_pointer_get_dx(self.0) }
    }

    /// Returns the y delta between the last event and the current event.
    ///
    /// If a device employs pointer acceleration, the delta returned by this function is the
    /// accelerated delta. Relative motion deltas are to be interpreted as pixel movement of a
    /// standardized mouse.
    #[inline]
    pub fn dy(&self) -> f64 {
        // It is an application bug to call this function for events other than
        // LIBINPUT_EVENT_POINTER_MOTION.
        unsafe { libinput_event_pointer_get_dy(self.0) }
    }

    /// Returns the unaccelerated x delta between the last event and the current event.
    ///
    /// Relative unaccelerated motion deltas are raw device coordinates. Note that these coordinates
    /// are subject to the device's native resolution. Touchpad coordinates represent raw device
    /// coordinates in the X resolution of the touchpad.
    ///
    /// Any rotation applied to the device also applies to unaccelerated motion (see
    /// libinput_device_config_rotation_set_angle()).
    #[inline]
    pub fn dx_unaccelerated(&self) -> f64 {
        unsafe { libinput_event_pointer_get_dx_unaccelerated(self.0) }
    }

    /// Returns the unaccelerated y delta between the last event and the current event.
    ///
    /// Relative unaccelerated motion deltas are raw device coordinates. Note that these coordinates
    /// are subject to the device's native resolution. Touchpad coordinates represent raw device
    /// coordinates in the X resolution of the touchpad.
    ///
    /// Any rotation applied to the device also applies to unaccelerated motion (see
    /// libinput_device_config_rotation_set_angle()).
    #[inline]
    pub fn dy_unaccelerated(&self) -> f64 {
        unsafe { libinput_event_pointer_get_dy_unaccelerated(self.0) }
    }

    /// Returns the current absolute x coordinate.
    ///
    /// Return the current absolute x coordinate of the pointer event, in mm from
    /// the top left corner of the device. To get the corresponding output screen
    /// coordinate, use [`Pointer::absolute_x_transformed`].
    #[inline]
    pub fn absolute_x(&self) -> f64 {
        // It is an application bug to call this function for events other than
        // LIBINPUT_EVENT_POINTER_MOTION_ABSOLUTE.
        unsafe { libinput_event_pointer_get_absolute_x(self.0) }
    }

    /// Returns the current absolute y coordinate.
    ///
    /// Return the current absolute y coordinate of the pointer event, in mm from
    /// the top left corner of the device. To get the corresponding output screen
    /// coordinate, use [`Pointer::absolute_y_transformed`].
    #[inline]
    pub fn absolute_y(&self) -> f64 {
        // It is an application bug to call this function for events other than
        // LIBINPUT_EVENT_POINTER_MOTION_ABSOLUTE.
        unsafe { libinput_event_pointer_get_absolute_y(self.0) }
    }

    /// Returns the current absolute x coordinate transformed to a screen coordinate
    ///
    /// `width` is the current output screen width.
    #[inline]
    pub fn absolute_x_transformed(&self, width: u32) -> f64 {
        // It is an application bug to call this function for events other than
        // LIBINPUT_EVENT_POINTER_MOTION_ABSOLUTE.
        unsafe { libinput_event_pointer_get_absolute_x_transformed(self.0, width) }
    }

    /// Returns the current absolute y coordinate transformed to a screen coordinate
    ///
    /// `height` is the current output screen height.
    #[inline]
    pub fn absolute_y_transformed(&self, height: u32) -> f64 {
        // It is an application bug to call this function for events other than
        // LIBINPUT_EVENT_POINTER_MOTION_ABSOLUTE.
        unsafe { libinput_event_pointer_get_absolute_y_transformed(self.0, height) }
    }

    /// Returns the button triggering this event.
    #[inline]
    pub fn button(&self) -> u32 {
        // It is an application bug to call this function for events other than
        // LIBINPUT_EVENT_POINTER_BUTTON.
        unsafe { libinput_event_pointer_get_button(self.0) }
    }

    /// Returns the button state triggering this event.
    #[inline]
    pub fn button_state(&self) -> ButtonState {
        // It is an application bug to call this function for events other than
        // LIBINPUT_EVENT_POINTER_BUTTON.
        unsafe { libinput_event_pointer_get_button_state(self.0) }
    }

    /// Returns non-zero if this event contains a value for this axis.
    ///
    /// If this function returns non-zero for an axis and [`Pointer::axis_value`] returns a value of
    /// 0, the event is a scroll stop event.
    #[inline]
    pub fn has_axis(&self, axis: Axis) -> i32 {
        // It is an application bug to call this function for events other than
        // LIBINPUT_EVENT_POINTER_AXIS,
        // LIBINPUT_EVENT_POINTER_SCROLL_WHEEL,
        // LIBINPUT_EVENT_POINTER_SCROLL_FINGER, or
        // LIBINPUT_EVENT_POINTER_SCROLL_CONTINUOUS.
        unsafe { libinput_event_pointer_has_axis(self.0, axis) }
    }

    /// Returns the axis value of this event.
    ///
    /// The interpretation of the value depends on the axis. For the two scrolling axes
    /// [`Axis::ScrollVertical`] and [`Axis::ScrollHorizontal`], the value of the event is in
    /// relative scroll units, with the positive direction being down or right, respectively. For
    /// the interpretation of the value, see [`Pointer::axis_source`].
    #[inline]
    pub fn axis_value(&self, axis: Axis) -> f64 {
        // It is an application bug to call this function for events other than
        // LIBINPUT_EVENT_POINTER_AXIS.
        unsafe { libinput_event_pointer_get_axis_value(self.0, axis) }
    }

    /// Returns the source for this axis event.
    #[inline]
    pub fn axis_source(&self) -> AxisSource {
        // It is an application bug to call this function for events other than
        // LIBINPUT_EVENT_POINTER_AXIS.
        unsafe { libinput_event_pointer_get_axis_source(self.0) }
    }

    /// Returns the axis value of the given axis.
    #[inline]
    pub fn scroll_value(&self, axis: Axis) -> f64 {
        // It is an application bug to call this function for events other than
        // LIBINPUT_EVENT_POINTER_SCROLL_WHEEL,
        // LIBINPUT_EVENT_POINTER_SCROLL_FINGER, or
        // LIBINPUT_EVENT_POINTER_SCROLL_CONTINUOUS.
        unsafe { libinput_event_pointer_get_scroll_value(self.0, axis) }
    }

    /// Returns the axis value of the given axis normalized to the 0-±120 range.
    #[inline]
    pub fn scroll_value_v120(&self, axis: Axis) -> f64 {
        // It is an application bug to call this function for events other than
        // LIBINPUT_EVENT_POINTER_SCROLL_WHEEL.
        unsafe { libinput_event_pointer_get_scroll_value_v120(self.0, axis) }
    }
}

// ===== enums =====

/// Logical state of a physical button.
///
/// Note that the logical state may not represent the physical state of the button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum ButtonState {
    Released = 0,
    Pressed = 1,
}

/// Axes on a device with the capability [`Capability::Pointer`] that are not x or y coordinates.
///
/// The two scroll axes are engaged separately, depending on the device. Libinput provides some
/// scroll direction locking but it is up to the caller to determine which axis is needed and
/// appropriate in the current interaction
///
/// [`Capability::Pointer`]: crate::ffi::input::Capability::Pointer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Axis {
    ScrollVertical = 0,
    ScrollHorizontal = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum AxisSource {
    /// The event is caused by the rotation of a wheel.
    Wheel = 1,
    /// The event is caused by the movement of one or more fingers on a device.
    Finger,
    /// The event is caused by the motion of some device.
    Continuous,
    /// The event is caused by the tilting of a mouse wheel rather than its rotation.
    ///
    /// This method is commonly used on mice without separate horizontal scroll wheels.
    ///
    /// This axis source is deprecated as of libinput 1.16. It was never used by any device before
    /// libinput 1.16. All wheel tilt devices use [`AxisSource::Wheel`] instead.
    WheelTilt,
}

// ===== ffi =====

type PointerPtr = NonNull<c_void>;

unsafe extern "C" {
    fn libinput_event_get_pointer_event(event: EventPtr) -> PointerPtr;
    fn libinput_event_pointer_get_time(pt: PointerPtr) -> u32;
    fn libinput_event_pointer_get_time_usec(pt: PointerPtr) -> u64;
    fn libinput_event_pointer_get_dx(pt: PointerPtr) -> c_double;
    fn libinput_event_pointer_get_dy(pt: PointerPtr) -> c_double;
    fn libinput_event_pointer_get_dx_unaccelerated(pt: PointerPtr) -> c_double;
    fn libinput_event_pointer_get_dy_unaccelerated(pt: PointerPtr) -> c_double;
    fn libinput_event_pointer_get_absolute_x(pt: PointerPtr) -> c_double;
    fn libinput_event_pointer_get_absolute_y(pt: PointerPtr) -> c_double;
    fn libinput_event_pointer_get_absolute_x_transformed(pt: PointerPtr, w: u32) -> c_double;
    fn libinput_event_pointer_get_absolute_y_transformed(pt: PointerPtr, h: u32) -> c_double;
    fn libinput_event_pointer_get_button(pt: PointerPtr) -> u32;
    fn libinput_event_pointer_get_button_state(pt: PointerPtr) -> ButtonState;
    fn libinput_event_pointer_get_base_event(pt: PointerPtr) -> EventPtr;

    fn libinput_event_pointer_has_axis(pt: PointerPtr, axis: Axis) -> c_int;
    fn libinput_event_pointer_get_axis_value(
        pt: PointerPtr,
        axis: Axis,
    ) -> c_double;
    fn libinput_event_pointer_get_axis_source(pt: PointerPtr) -> AxisSource;
    fn libinput_event_pointer_get_scroll_value(
        pt: PointerPtr,
        axis: Axis,
    ) -> c_double;
    fn libinput_event_pointer_get_scroll_value_v120(
        pt: PointerPtr,
        axis: Axis,
    ) -> c_double;

    // This function does not support high-resolution mouse wheels and should be considered
    // deprecated as of libinput 1.19.
    // fn libinput_event_pointer_get_axis_value_discrete(
    //     pt: PointerPtr,
    //     axis: Axis,
    // ) -> c_double;
}
