use crate::wayland::Interface;

/// Type that is associated with an interface.
pub trait AsInterface {
    /// Returns the interface that this type associated with.
    fn interface(&self) -> Interface;
}
