//! # Data Type
//!
//! - `int`: `i32`
//!
//! 32-bit signed integer.
//!
//! - `uint`: `u32`
//!
//! 32-bit unsigned integer.
//!
//! - `fixed`: `f32`
//!
//! Signed 24.8 decimal numbers. It is a signed decimal type which offers a sign bit, 23 bits of
//! integer precision and 8 bits of decimal precision.
//!
//! - `string`: `str`
//!
//! Starts with an unsigned 32-bit length (including null terminator), followed by the UTF-8 encoded
//! string contents, including terminating null byte, then padding to a 32-bit boundary. A null
//! value is represented with a length of 0. Interior null bytes are not permitted.
//!
//! - `object`: [`ObjectId`]
//!
//! 32-bit object ID. A null value is represented with an ID of 0.
//!
//! - `new_id`: [`NewId`], `new_id<iface>`: [`NewIdOf<Iface>`]
//!
//! The 32-bit object ID. Generally, the interface used for the new object is inferred from the xml,
//! but in the case where it’s not specified, a new_id is preceded by a string specifying the
//! interface name, and a uint specifying the version.
//!
//! - `array`: `[u8]`
//!
//! Starts with 32-bit array size in bytes, followed by the array contents verbatim, and finally
//! padding to a 32-bit boundary.
//!
//! - `fd`: `std::os::fd::RawFd`;
//!
//! The file descriptor is not stored in the message buffer, but in the ancillary data of the UNIX
//! domain socket message (msg_control).
