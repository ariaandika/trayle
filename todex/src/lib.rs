//! # The Torio Project Core Module.
//!
//! This crate provide low level system calls, generic collections, wayland protocol coding, shared
//! abstraction and high level state management.
//!
//! # Event Loop
//!
//! This crate does not provide an event loop.
//!
//! [`Poller`][poller::Poller] monitor resources for readiness. Application can register sources,
//! like listener or socket, then `Poller` will callback with an events ready to proceed. See its
//! documentation for more details.
//!
//! # Wayland
//!
//! [`wayland`] module provide protocol definition, message wire format, and wayland primitive
//! types. See its documentation for more details.
//!
//! # State Management
//!
//! This crate provide high level state management that can be used by wayland server or client.
//!
//! # Collection and Allocation
//!
//! Application usually does not interact with data structures or allocation directly, instead uses
//! high level APIs mentioned previously. But for simple cases, [`collections`] module provide
//! generic data structures.
#![allow(clippy::new_without_default, clippy::module_inception)]
#![warn(
    clippy::allow_attributes/* _without_reason */,
    clippy::option_if_let_else,
    clippy::equatable_if_let,
    clippy::let_underscore_untyped,
)]

pub mod bitflags;

pub mod sys;
pub mod alloc;
pub mod collections;
pub mod wayland;
pub mod compositor;
pub mod poller;
pub mod log;
