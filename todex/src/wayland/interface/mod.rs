#![expect(clippy::too_many_arguments)]
use std::os::fd::RawFd;
use macros::interface;

use crate::wayland::primitives::*;
use crate::wayland::object::{AsNewId, NewId, Object, WlGlobal};
use crate::wayland::wire::{AsOpCode, OpCode};
use crate::wayland::wire::{DecodeError, DecodePayload, Reader};
use crate::wayland::wire::{EncodePayload, Sized2, Writer};
use crate::wayland::message::{Message, WlMessage};
use crate::wayland::display;

// ===== marker =====

macro_rules! assert_iface {
    ($i:ident, $ty:ident) => {
        if !matches!($i, InterfaceId::$ty) {
            invalid_interface();
        }
    };
}

#[cold]
#[inline(never)]
fn invalid_interface() {
    panic!("unchecked interface marker creation")
}

pub(crate) mod sealed {
    /// Internal usage only
    pub trait Sealed: std::fmt::Debug + Clone + Copy {
        /// Caller must ensure correctness for creating this marker.
        const MARKER: Self;
    }

    impl Sealed for () {
        const MARKER: Self = ();
    }
}

pub trait InterfaceMarker: sealed::Sealed {
    /// Create this interface marker.
    ///
    /// # Panics
    ///
    /// Panics if the interface type does not match with given [`Interface`].
    fn from_interface(interface: Interface) -> Self;
}

impl InterfaceMarker for () {
    #[inline]
    fn from_interface(_: Interface) -> Self { }
}

// ===== interface =====

pub trait WlInterface: AsInterface {
    type RequestOp: OpCode;

    type EventOp: OpCode;
}

/// Type that is associated with an interface.
///
/// This is utilized by type that wraps either a type safe interface or runtime value
/// [`InterfaceId`].
pub trait AsInterface {
    /// Returns the interface that this type associated with.
    fn interface(&self) -> Interface;
}

impl<I: AsInterface> AsInterface for &I {
    #[inline]
    fn interface(&self) -> Interface {
        I::interface(self)
    }
}

impl AsInterface for Interface {
    #[inline]
    fn interface(&self) -> Interface {
        *self
    }
}

// ===== wl_display =====

pub use display_id::DisplayId;

mod display_id;

// ===== protocols =====

pub use wl_core::*;
pub use xdg_shell::*;

mod wl_core;
mod xdg_shell;

// ===== protocols =====

pub use Interface as InterfaceId;

macros::interface_id! {
    /// Reexport interface modules as upper camel case.
    pub mod camel_cased;

    /// Interface definitions id.
    pub enum Interface;

    WlDisplay,
    WlRegistry,
    WlCallback,
    WlCompositor,
    WlShmPool,
    WlShm,
    WlBuffer,
    WlDataOffer,
    WlDataSource,
    WlDataDevice,
    WlDataDeviceManager,
    WlSurface,
    WlSeat,
    WlPointer,
    WlKeyboard,
    WlTouch,
    WlOutput,
    WlRegion,
    WlSubcompositor,
    WlSubsurface,
    // WlFixes,
    XdgWmBase,
    XdgPositioner,
    XdgSurface,
    XdgToplevel,
    XdgPopup,
    // ZwpLinuxDmabufV1,
    // ZwpLinuxBufferParams_v1,
    // ZwpLinuxDmabufFeedbackV1,
}
