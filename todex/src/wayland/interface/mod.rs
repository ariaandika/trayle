#![expect(clippy::too_many_arguments)]
use std::os::fd::RawFd;
use macros::interface;

use crate::wayland::primitives::*;
use crate::wayland::object::{AsNewId, NewId, Object, WlGlobal};
use crate::wayland::display::{self, FieldDisplay, FormatterExt};
use crate::wayland::message::{AsOpCode, Message, OpCode, WlMessage};
use crate::wayland::wire::{DecodeError, DecodePayload, Reader};
use crate::wayland::wire::{EncodePayload, Sized2, Writer};

// ===== marker =====

pub(crate) mod sealed {
    /// Internal usage only
    pub trait Sealed: std::fmt::Debug + Clone + Copy {
        /// Caller must ensure correctness for creating this marker.
        const MARKER: Self;
    }
}

// ===== interface =====

/// Wayland interface.
///
/// This trait is implemented by a marker type.
pub trait WlInterface: Sized + Copy + AsInterface + sealed::Sealed {
    type RequestOp: OpCode;

    type EventOp: OpCode;

    /// Interface name.
    const INTERFACE_NAME: &str;

    /// Create this interface.
    ///
    /// Returns `None` if the interface type does not match with given [`Interface`].
    fn try_from_interface(interface: Interface) -> Option<Self>;

    /// Create this interface marker.
    ///
    /// # Panics
    ///
    /// Panics if the interface type does not match with given [`Interface`].
    #[inline]
    fn from_interface(interface: Interface) -> Self {
        #[cold]
        #[inline(never)]
        fn invalid_interface() -> ! {
            panic!("unchecked interface marker creation")
        }
        Self::try_from_interface(interface).unwrap_or_else(|| invalid_interface())
    }
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

// ===== protocols =====

pub use wl_core::*;
pub use xdg_shell::*;
pub use wl_display::WlDisplay;

mod wl_core;
mod xdg_shell;
pub mod wl_display;

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
