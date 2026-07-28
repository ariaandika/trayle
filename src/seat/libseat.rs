#![expect(non_camel_case_types)]
use std::ffi::c_void;
use std::os::fd::{AsFd, BorrowedFd};
use std::ptr::NonNull;

use todex::log;
use todex::sys::error::ErrCode;

// ===== Libseat =====

/// Seat management via `libseat`.
///
/// # Known Issue
///
/// `Ctrl + Alt + F[N]` does not trigger vt change, but seat togling logic works fine (test using
/// chvt)
pub struct Libseat {
    seat: NonNull<libseat>,
}

impl Drop for Libseat {
    fn drop(&mut self) {
        unsafe { libseat_close_seat(self.seat.as_ptr()) };
    }
}

impl Libseat {
    pub fn new() -> Self {
        let seat = unsafe { libseat_open_seat((&raw const LISTENER).cast_mut(), 0 as _) };
        let Some(seat) = NonNull::new(seat).map(|seat|Self { seat }) else {
            panic!("cannot open seat: {}", ErrCode::errno());
        };
        // initial dispatch, based on the example in seatd repo
        seat.dispatch_inner(-1);
        seat
    }

    pub fn dispatch(&self) {
        log::info!("dispatched");
        self.dispatch_inner(0);
    }

    fn dispatch_inner(&self, timeout: i32) {
        let result = unsafe { libseat_dispatch(self.seat.as_ptr(), timeout) };
        if result == -1 {
            panic!("failed to dispatch libseat: {}", ErrCode::errno());
        }
    }
}

impl AsFd for Libseat {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(libseat_get_fd(self.seat.as_ptr())) }
    }
}

// ===== callbacks =====

extern "C" fn enable_seat(_seat: *mut libseat, _userdata: *mut c_void) {
    log::info!(target: "libseat", "seat enabled");
}

extern "C" fn disable_seat(seat: *mut libseat, _userdata: *mut c_void) {
    log::info!(target: "libseat", "seat disabled");
    let result = unsafe { libseat_disable_seat(seat) };
    if result == -1 {
        log::error!(target: "libseat", "failed to ack seat disable: {}", ErrCode::errno());
    };
}

static LISTENER: libseat_seat_listener = libseat_seat_listener {
    enable_seat,
    disable_seat,
};

// ===== ffi =====

enum libseat {}

#[repr(C)]
struct libseat_seat_listener {
    /// The seat has been enabled, and is now valid for use. Re-open all seat
    /// devices to ensure that they are operational, as existing fds may have
    /// had their functionality blocked or revoked.
    enable_seat: extern "C" fn(seat: *mut libseat, userdata: *mut c_void),
    /// The seat has been disabled. This event signals that the application
    /// is going to lose its seat access. The event *must* be acknowledged
    /// with libseat_disable_seat shortly after receiving this event.
    ///
    /// If the recepient fails to acknowledge the event in time, seat devices
    /// may be forcibly revoked by the seat provider.
    disable_seat: extern "C" fn(seat: *mut libseat, userdata: *mut c_void),
}

#[link(name = "seat")]
unsafe extern "C" {
    /// Opens a seat, taking control of it if possible and returning a pointer to
    /// the libseat instance. If LIBSEAT_BACKEND is set, the specified backend is
    /// used. Otherwise, the first successful backend will be used.
    ///
    /// The seat listener specified is used to signal events on the seat, and must
    /// be non-NULL. The userdata pointer will be provided in all calls to the seat
    /// listener.
    ///
    /// The available backends, if enabled at compile-time, are: seatd, logind and
    /// builtin.
    ///
    /// To use builtin, the process must have permission to open and use the seat's
    /// devices at the time of the call. In the case of DRM devices, this includes
    /// permission for drmSetMaster(3). These privileges can be dropped at any
    /// point after the call.
    ///
    /// The returned pointer must be destroyed with libseat_close_seat.
    ///
    /// Returns a pointer to an opaque libseat struct on success. Returns NULL and
    /// sets errno on error.
    fn libseat_open_seat(
        listener: *mut libseat_seat_listener,
        userdata: *mut c_void,
    ) -> *mut libseat;

    fn libseat_close_seat(seat: *mut libseat) -> i32;

    fn libseat_disable_seat(seat: *mut libseat) -> i32;

    /// Retrieve the pollable connection fd for a given libseat instance. Used to
    /// poll the libseat connection for events that need to be dispatched.
    ///
    /// Returns a pollable fd on success. Returns -1 and sets errno on error.
    fn libseat_get_fd(seat: *mut libseat) -> i32;

    /// Reads and dispatches events on the libseat connection fd.
    ///
    /// The specified timeout dictates how long libseat might wait for data if none
    /// is available: 0 means that no wait will occur, -1 means that libseat might
    /// wait indefinitely for data to arrive, while > 0 is the maximum wait in
    /// milliseconds that might occur.
    ///
    /// Returns a positive number signifying processed internal messages on success.
    /// Returns 0 if no messages were processed. Returns -1 and sets errno on error.
    fn libseat_dispatch(seat: *mut libseat, timeout: i32) -> i32;
}
