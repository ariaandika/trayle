use crate::wayland::{WlObject, Interface};

/// Type that is associated with an interface.
pub trait AsInterface {
    /// Returns the interface that this type associated with.
    fn interface(&self) -> Interface;
}

impl AsInterface for Interface {
    #[inline]
    fn interface(&self) -> Interface {
        *self
    }
}

pub trait WlInterface: Sized + WlObject + AsInterface { }

impl<O: WlObject + AsInterface> WlInterface for O { }
