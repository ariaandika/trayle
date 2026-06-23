use crate::wayland::primitives::Version;
use crate::wayland::{Interface, Object};

pub type Global = Object<Interface, Version, &'static str>;

/// Create global [`Object`] for specified interface.
pub const fn global_of<G: WlGlobal>() -> Global {
    Object::from_parts(G::INTERFACE, G::VERSION, G::NAME)
}

/// Type that is a wayland global object.
pub trait WlGlobal {
    /// Interface name.
    const NAME: &str;

    /// Object version.
    const VERSION: Version;

    /// [`Interface`] value for this interface.
    const INTERFACE: Interface;

    /// Create global [`Object`] for this interface.
    #[inline]
    fn global_object() -> Global {
        Object::from_parts(Self::INTERFACE, Self::VERSION, Self::NAME)
    }
}
