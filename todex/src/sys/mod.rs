//! OS APIs and ffi.
pub mod error;

// ===== syscall =====

pub mod bytes;
pub mod cmsg;
pub mod memmap;

pub mod socket;
pub mod listener;
pub mod sigfd;
pub mod memfd;
pub mod epoll;

// ===== ffi =====
mod macros;
pub mod xkb;
pub mod udev;
pub mod libseat;
