use crate::wayland::{WlObject, Interface};

/// Type that is associated with an interface.
pub trait AsInterface {
    /// Returns the interface that this type associated with.
    fn interface(&self) -> Interface;
}
pub trait InterfaceMarker: sealed::Sealed {}

impl AsInterface for Interface {
    #[inline]
    fn interface(&self) -> Interface {
        *self
    }
mod sealed {
    pub trait Sealed: std::fmt::Debug + Default + Clone + Copy {}
}

pub trait WlInterface: Sized + WlObject + AsInterface { }

impl<O: WlObject + AsInterface> WlInterface for O { }

// ===== protocols =====

use crate::wayland::prelude::*;

pub use wl_core::*;

mod wl_core;
