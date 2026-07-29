use std::ffi::{CStr, c_char, c_void};
use std::os::fd::{AsFd, BorrowedFd, FromRawFd, OwnedFd};
use std::ptr::NonNull;

use crate::sys::error::{ErrCode, ResCode, OsError, simple_os_error};

// ===== Listener =====

/// Seat event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeatEvent {
    /// The seat has been enabled.
    Enable,
    /// The seat has been disabled.
    Disable,
}

/// Seat event listener.
pub trait Listener {
    /// Dispatch seat event.
    ///
    /// If event is [`SeatEvent::Enable`], the seat has been enabled, and is now valid for use.
    /// Re-open all seat devices to ensure that they are operational, as existing fds may have had
    /// their functionality blocked or revoked.
    ///
    /// If event is [`SeatEvent::Disable`], the seat has been disabled. This event signals that the
    /// application is going to lose its seat access. The event *must* be acknowledged with
    /// [`Context::disable_seat`] shortly after receiving this event.
    ///
    /// If the recepient fails to acknowledge the disable event in time, seat devices may be
    /// forcibly revoked by the seat provider.
    fn seat_event(&mut self, event: SeatEvent, seat: &mut Context);
}

// ===== Context =====

/// Libseat context.
#[repr(transparent)]
pub struct Context(ContextPtr);

impl Context {
    /// Reads and dispatches events on the libseat connection fd.
    ///
    /// Returns 0 if no messages were processed.
    ///
    /// The specified timeout dictates how long libseat might wait for data if none is available:
    /// `0` means that no wait will occur, `-1` means that libseat might wait indefinitely for data
    /// to arrive, while `> 0` is the maximum wait in milliseconds that might occur.
    #[inline]
    pub fn dispatch(&mut self, timeout: i32) -> Result<u32, DispatchError> {
        // Returns a positive number signifying processed internal messages on success.
        // Returns -1 and sets errno on error.
        unsafe { libseat_dispatch(self.0, timeout) }.uint()
    }

    /// Disables a seat, used in response to a [`SeatEvent::Disable`].
    ///
    /// After disabling the seat, the seat devices must not be used until [`SeatEvent::Enable`] is
    /// received, and all requests on the seat will fail during this period.
    #[inline]
    pub fn disable_seat(&self) -> Result<(), AckDisableError> {
        unsafe { libseat_disable_seat(self.0) }.ok()
    }

    /// Opens a device on the seat.
    ///
    /// This will only succeed if the seat is active and the device is of a type permitted for
    /// opening on the backend, such as drm and evdev.
    ///
    /// The device may be revoked in some situations, such as in situations where a seat session
    /// switch is being forced.
    #[inline]
    pub fn open_device(&self, path: &CStr) -> Result<Device, DeviceError> {
        let mut fd = 0;
        unsafe { libseat_open_device(self.0, path.as_ptr(), &mut fd) }
            .uint()
            .map(|id: u32| Device {
                id: id as i32,
                fd: unsafe { OwnedFd::from_raw_fd(fd) },
            })
    }

    /// Closes a device that has been opened on the seat.
    #[inline]
    pub fn close_device(&self, device: Device) {
        unsafe { libseat_close_device(self.0, device.id) };
    }

    /// Retrieves the name of the seat that is currently made available through the provided libseat
    /// instance.
    #[inline]
    pub fn name(&self) -> &CStr {
        // The returned string is owned by the libseat instance, and must not be
        // modified. It remains valid as long as the seat is open.
        unsafe { CStr::from_ptr(libseat_seat_name(self.0)) }
    }

    /// Requests that the seat switches session to the specified session number.
    ///
    /// For seats that are VT-bound, the session number matches the VT number, and switching session
    /// results in a VT switch.
    ///
    /// This call does not imply that a switch will occur, and the caller should assume that the
    /// session continues unaffected.
    #[inline]
    pub fn switch_session(&self, session: i32) -> Result<(), SwitchSessionError> {
        unsafe { libseat_switch_session(self.0, session) }.ok()
    }

    /// Retrieve the pollable connection fd for a given libseat instance. Used to poll the libseat
    /// connection for events that need to be dispatched.
    ///
    /// # Panics
    ///
    /// Panics if `libseat` cannot get the fd.
    #[inline]
    pub fn get_fd(&self) -> BorrowedFd<'_> {
        unsafe {
            let fd = libseat_get_fd(self.0);
            if fd == -1 {
                panic!("failed to get libseat seat fd: {}", ErrCode::errno());
            }
            BorrowedFd::borrow_raw(fd)
        }
    }
}

impl AsFd for Context {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.get_fd()
    }
}

// ===== Libseat =====

pub struct Libseat {
    context: Context,
    /// Libseat context does not provide getter for userdata, thus the data pointer need to be kept
    /// for cleeanup
    #[expect(dead_code)]
    adapter: Adapter,
}

impl std::ops::Deref for Libseat {
    type Target = Context;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl std::ops::DerefMut for Libseat {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

impl Drop for Libseat {
    #[inline]
    fn drop(&mut self) {
        unsafe { libseat_close_seat(self.context.0) };
    }
}

impl Libseat {
    /// Opens a seat, taking control of it if possible.
    ///
    /// If `LIBSEAT_BACKEND` is set, the specified backend is used. Otherwise, the first successful
    /// backend will be used.
    ///
    /// The available backends, if enabled at compile-time, are: `seatd`, `logind` and `builtin`.
    ///
    /// To use `builtin`, the process must have permission to open and use the seat's devices at the
    /// time of the call. In the case of DRM devices, this includes permission for
    /// `drmSetMaster(3)`. These privileges can be dropped at any point after the call.
    pub fn open<L: Listener>(listener: L) -> Result<Libseat, ContextError> {
        Self::open_inner(Adapter::new(listener))
    }

    #[inline]
    fn open_inner(adapter: Adapter) -> Result<Self, ContextError> {
        adapter.open_seat().map_or_else(
            || Err(<_>::errno()),
            |context| Ok(Self { context, adapter }),
        )
    }
}

impl AsFd for Libseat {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.context.get_fd()
    }
}

// ===== Device =====

/// An owned open device.
///
/// This closes the device on drop. Although, caller might need to release the device ownership via
/// [`Libseat::close_device`].
#[derive(Debug)]
pub struct Device {
    id: i32,
    fd: OwnedFd,
}

impl AsFd for Device {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

// ===== Adapter =====

struct Adapter(NonNull<AdapterData>);

impl Drop for Adapter {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let drop = self.0.as_ref().drop;
            drop(self.0);
        }
    }
}

impl Adapter {
    fn new<L: Listener>(listener: L) -> Self {
        Self(
            NonNull::new(Box::into_raw(Box::new(AdapterData::new(listener))).cast())
                .expect("box is non-null"),
        )
    }

    fn open_seat(&self) -> Option<Context> {
        unsafe { libseat_open_seat((&raw const LISTENER).cast_mut(), self.0.as_ptr().cast()) }
    }
}

#[repr(C)]
struct AdapterData<T = ()> {
    seat_event: fn(*mut (), SeatEvent, &mut Context),
    /// drop self with the correct generic type
    drop: fn(NonNull<AdapterData<()>>),
    data: T,
}

impl<L> AdapterData<L> {
    fn new(listener: L) -> Self where L: Listener {
        Self {
            seat_event: |data, event, seat| unsafe {
                L::seat_event(data.cast::<L>().as_mut_unchecked(), event, seat)
            },
            drop: |data| unsafe { drop(Box::from_raw(data.cast::<Self>().as_ptr())) },
            data: listener,
        }
    }
}

#[repr(C)]
struct libseat_seat_listener {
    enable_seat: extern "C" fn(ContextPtr, userdata: NonNull<AdapterData>),
    disable_seat: extern "C" fn(ContextPtr, userdata: NonNull<AdapterData>),
}

static LISTENER: libseat_seat_listener = {
    extern "C" fn enable_seat(seat: ContextPtr, mut data: NonNull<AdapterData>) {
        unsafe {
            let me = data.as_mut();
            (me.seat_event)(&raw mut me.data, SeatEvent::Enable, &mut Context(seat));
        }
    }
    extern "C" fn disable_seat(seat: ContextPtr, mut data: NonNull<AdapterData>) {
        unsafe {
            let me = data.as_mut();
            (me.seat_event)(&raw mut me.data, SeatEvent::Disable, &mut Context(seat));
        }
    }
    libseat_seat_listener {
        enable_seat,
        disable_seat,
    }
};

// ===== error =====

/// An error that can occur during libseat context creation.
#[derive(Clone, Copy)]
pub struct ContextError(ErrCode);

simple_os_error!(ContextError, "create libseat context");

/// An error that can occur during libseat event dispatching.
#[derive(Clone, Copy)]
pub struct DispatchError(ErrCode);

simple_os_error!(DispatchError, "dispatch libseat event");

/// An error that can occur during libseat disable event acknowledgement.
#[derive(Clone, Copy)]
pub struct AckDisableError(ErrCode);

simple_os_error!(AckDisableError, "acknowledge libseat disable event");

/// An error that can occur during opening libseat device.
#[derive(Clone, Copy)]
pub struct DeviceError(ErrCode);

simple_os_error!(DeviceError, "open libseat device");

/// An error that can occur during libseat session switch request.
#[derive(Clone, Copy)]
pub struct SwitchSessionError(ErrCode);

simple_os_error!(SwitchSessionError, "request libseat session switch");

// ===== ffi =====

// https://git.sr.ht/~kennylevinsen/seatd
//
// include/libseat.h

type ContextPtr = NonNull<c_void>;

unsafe extern "C" {
    fn libseat_open_seat(
        listener: *mut libseat_seat_listener,
        userdata: *mut c_void,
    ) -> Option<Context>;
    fn libseat_disable_seat(seat: ContextPtr) -> ResCode;
    fn libseat_close_seat(seat: ContextPtr) -> i32;
    fn libseat_open_device(seat: ContextPtr, path: *const c_char, fd: *mut i32) -> ResCode;
    fn libseat_close_device(seat: ContextPtr, device_id: i32) -> ResCode;
    fn libseat_seat_name(seat: ContextPtr) -> *const c_char;
    fn libseat_switch_session(seat: ContextPtr, session: i32) -> ResCode;
    fn libseat_get_fd(seat: ContextPtr) -> i32;
    fn libseat_dispatch(seat: ContextPtr, timeout: i32) -> ResCode;
}
