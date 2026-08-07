//! OS APIs and ffi.
pub mod error;

// ===== syscall =====

pub mod socket;
pub mod listener;
pub mod sigfd;
pub mod memfd;
pub mod epoll;

// ===== ffi =====

mod macros;

pub mod udev;
pub mod libseat;

// ===== input =====

pub mod keycode;
pub mod xkb;
pub mod libinput;
