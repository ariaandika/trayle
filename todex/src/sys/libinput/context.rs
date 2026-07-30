use std::ffi::{CStr, c_char, c_void};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};
use std::ptr::NonNull;

use crate::sys::error::{ErrCode, OsError, ResCode, simple_os_error};
use crate::sys::libinput::{Event, EventKind, Interface};
use crate::sys::libinput::interface::Adapter;
use crate::sys::macros::simple_ffi;
use crate::sys::udev::Udev;

// ===== Libinput =====

/// Libinput context.
///
/// This struct only provide context with udev integration.
#[repr(transparent)]
pub struct Libinput(InputPtr);

impl Drop for Libinput {
    fn drop(&mut self) {
        unsafe {
            Adapter::drop(libinput_get_user_data(self.0));
            libinput_unref(self.0);
        }
    }
}

simple_ffi!(impl Clone for Libinput::libinput_ref);
simple_ffi!(impl Debug for Libinput);

impl Libinput {
    /// Create a new libinput context from udev with `libc::open` and `libc::close` implementation.
    ///
    /// This context is inactive until assigned a seat ID with [`Input::assign_seat`].
    #[inline]
    pub fn new_libc(udev: &Udev) -> Result<Self, ContextError> {
        Self::udev_create_context(Adapter::new_libc(), udev)
    }

    /// Create a new libinput context from udev.
    ///
    /// This context is inactive until assigned a seat ID with [`Input::assign_seat`].
    #[inline]
    pub fn new<T: Interface>(interface: T, udev: &Udev) -> Result<Self, ContextError> {
        Self::udev_create_context(Adapter::new_boxed(interface), udev)
    }

    fn udev_create_context(adapter: Adapter, udev: &Udev) -> Result<Self, ContextError> {
        unsafe {
            libinput_udev_create_context(adapter.vtable(), adapter.data_ptr(), udev.as_ptr().cast())
                .ok_or_else(<_>::errno)
        }
    }

    pub(crate) fn as_ptr(&self) -> InputPtr {
        self.0
    }

    /// Assign a seat to this libinput context.
    ///
    /// New devices or the removal of existing devices will appear as events during
    /// [`Input::dispatch`].
    ///
    /// This method will succeeds even if no input devices are currently available on this seat, or
    /// if devices are available but fail to open in [`Interface::open_restricted`]. Devices that do
    /// not have the minimum capabilities to be recognized as pointer, keyboard or touch device are
    /// ignored. Such devices and those that failed to open are ignored until the next call to
    /// [`Input::resume`].
    ///
    /// This function may only be called once per context.
    #[inline]
    pub fn assign_seat(&mut self, seat_id: &CStr) -> Result<(), SeatError> {
        // return 0 on success or -1 on failure.
        unsafe { libinput_udev_assign_seat(self.0, seat_id.as_ptr()).ok() }
    }

    /// Returns the file descriptor used to notify of pending events.
    ///
    /// Libinput keeps a single file descriptor for all events. Call into [`Input::dispatch`] if any
    /// events become available on this fd.
    #[inline]
    pub fn fd(&self) -> BorrowedFd<'_> {
        unsafe { libinput_get_fd(self.0) }
    }

    /// Main event dispatchment function.
    ///
    /// Reads events of the file descriptors and processes them internally. Use [`Input::get_event`]
    /// to retrieve the events.
    ///
    /// Dispatching does not necessarily queue libinput events. This function should be called
    /// immediately once data is available on the file descriptor returned by [`Input::get_fd`].
    /// libinput has a number of timing-sensitive features (e.g. tap-to-click), any delay in calling
    /// [`Input::dispatch`] may prevent these features from working correctly.
    #[inline]
    pub fn dispatch(&mut self) -> Result<(), DispatchError> {
        // return 0 on success, or a negative errno on failure
        unsafe { libinput_dispatch(self.0) }.result()
    }

    /// Returns the next available event, or `None` if no event is available.
    #[inline]
    pub fn pop_event(&mut self) -> Option<Event> {
        Event::get_event(self)
    }

    /// Returns the event kind of the next available event or `None` if no event is available.
    ///
    /// This function does not pop the event off the queue and the next call to [`Input::pop_event`]
    /// returns that event.
    #[inline]
    pub fn event_kind(&self) -> Option<EventKind> {
        Event::peek_event_kind(self)
    }

    /// Suspend monitoring for new devices and close existing devices.
    ///
    /// This all but terminates libinput but does keep the context valid to be resumed with
    /// [`Input::resume`].
    #[inline]
    pub fn suspend(&mut self) {
        unsafe { libinput_suspend(self.0) };
    }

    /// Resume a suspended libinput context.
    ///
    /// This re-enables device monitoring and adds existing devices.
    #[inline]
    pub fn resume(&mut self) -> Result<(), ResumeError> {
        unsafe { libinput_resume(self.0) }.result()
    }
}

impl AsFd for Libinput {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd()
    }
}

impl AsRawFd for Libinput {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.fd().as_raw_fd()
    }
}

// ===== error =====

/// An error that can occur during libinput context creation.
#[derive(Clone, Copy)]
pub struct ContextError(ErrCode);

simple_os_error!(ContextError, "create libinput context");

/// An error that can occur during libinput seat assignment.
#[derive(Clone, Copy)]
pub struct SeatError(ErrCode);

simple_os_error!(SeatError, "assign libinput seat");

/// An error that can occur during libinput event dispatching.
#[derive(Clone, Copy)]
pub struct DispatchError(ErrCode);

simple_os_error!(DispatchError, "dispatch libinput event");

/// An error that can occur during libinput context resuming.
#[derive(Clone, Copy)]
pub struct ResumeError(ErrCode);

simple_os_error!(ResumeError, "resume libinput context");

// ===== ffi =====

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct InputPtr(NonNull<c_void>);

unsafe extern "C" {
    fn libinput_udev_create_context(
        interface: *const c_void,
        userdata: *mut c_void,
        udev: NonNull<c_void>,
    ) -> Option<Libinput>;
    fn libinput_ref(libinput: InputPtr) -> InputPtr;
    fn libinput_unref(libinput: InputPtr) -> InputPtr;

    fn libinput_udev_assign_seat(libinput: InputPtr, seat_id: *const c_char) -> ResCode;
    fn libinput_dispatch(libinput: InputPtr) -> ResCode;
    fn libinput_get_fd<'a>(libinput: InputPtr) -> BorrowedFd<'a>;

    // fn libinput_set_user_data(libinput: InputPtr, user_data: *mut c_void);
    fn libinput_get_user_data(libinput: InputPtr) -> *mut c_void;

    fn libinput_suspend(libinput: InputPtr);
    fn libinput_resume(libinput: InputPtr) -> ResCode;
}
