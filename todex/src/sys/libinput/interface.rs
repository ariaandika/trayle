//! The `libinput_interface` struct.
//!
//! Libinput accept vtable style callback arguments.
//!
//! Converting it to rust api:
//!
//! 1. Define context with lifetime tied to the vtable
//!     - not preferred
//! 2. Define context with generic containing the vtable
//!     - not preferred
//! 3. Accept static vtable
//!     - good for use case that does not requires states, or uses global state
//! 4. Allocate the vtable and the state to the heap
//!     - very flexible
//!
//! [`Adapter`] provide:
//!
//! - #2 that forwards call to `libc`.
//! - #3 with [`Adapter`].
use std::ffi::{CStr, c_char, c_void};
use std::os::fd::RawFd;
use std::ptr::{self, NonNull};

// ===== Interface =====

/// Correspond to `libinput_interface`.
pub trait Interface {
    fn open_restricted(&mut self, path: &CStr, flags: i32) -> i32;

    fn close_restricted(&mut self, fd: RawFd);
}

// ===== Adapter =====

pub struct Adapter(NonNull<Inner>);

#[repr(C)]
struct Inner<T = ()> {
    iface: libinput_interface,
    drop_in_place: unsafe fn(*mut T),
    data: T,
}

impl Adapter {
    pub(crate) fn new_libc() -> Self {
        Self(impl_libc::new())
    }

    pub(crate) fn new_boxed<T: Interface>(data: T) -> Self {
        Self(impl_boxed::new(data))
    }

    pub(crate) fn drop(data: *mut c_void) {
        unsafe {
            let Some(me) = data.cast::<Inner>().as_mut() else {
                return;
            };
            (me.drop_in_place)(&mut me.data);
        }
    }

    pub(crate) const fn vtable(&self) -> *const c_void {
        unsafe { &raw const (*self.0.as_ptr()).iface }.cast()
    }

    pub(crate) const fn data_ptr(&self) -> *mut c_void {
        self.0.as_ptr().cast()
    }
}

// ===== impl libc =====

mod impl_libc {
    use super::*;

    pub fn new() -> NonNull<Inner> {
        static IMPL: Inner = Inner {
            iface: libinput_interface {
                open_restricted,
                close_restricted,
            },
            drop_in_place: ptr::drop_in_place::<()>,
            data: (),
        };
        NonNull::from_ref(&IMPL)
    }

    extern "C" fn open_restricted(path: *const i8, fd: i32, _: *mut c_void) -> i32 {
        unsafe { libc::open(path, fd) }
    }

    extern "C" fn close_restricted(fd: RawFd, _: *mut c_void) {
        let _ = unsafe { libc::close(fd) };
    }
}

// ===== impl boxed =====

mod impl_boxed {
    use super::*;

    pub fn new<T: Interface>(data: T) -> NonNull<Inner> {
        NonNull::new(Box::into_raw(Box::new(Inner {
            iface: libinput_interface {
                open_restricted: open_restricted::<T>,
                close_restricted: close_restricted::<T>,
            },
            drop_in_place: ptr::drop_in_place::<T>,
            data,
        })))
        .expect("box is non-null")
        .cast()
    }

    extern "C" fn open_restricted<T: Interface>(
        path: *const c_char,
        flags: i32,
        data: *mut c_void,
    ) -> i32 {
        unsafe {
            data.cast::<T>()
                .as_mut_unchecked()
                .open_restricted(CStr::from_ptr(path), flags)
        }
    }

    extern "C" fn close_restricted<T: Interface>(fd: RawFd, data: *mut c_void) {
        unsafe { data.cast::<T>().as_mut_unchecked().close_restricted(fd) }
    }
}

// ===== ffi =====

#[repr(C)]
struct libinput_interface {
    /// Open the device at the given path with the flags provided and return the fd.
    open_restricted: extern "C" fn(path: *const c_char, flags: i32, data: *mut c_void) -> i32,
    /// Close the file descriptor.
    close_restricted: extern "C" fn(fd: RawFd, data: *mut c_void),
}
