use crate::wayland::{Interface, Version};

// ===== trait =====

/// Type that is a wayland global object.
pub trait WlGlobal {
    /// Interface name.
    const NAME: &str;

    /// Object version.
    const VERSION: Version;

    /// [`Interface`] value for this interface.
    const INTERFACE: Interface;

    /// Create [`Global`] object for this interface.
    #[inline]
    fn global() -> Global {
        Global {
            name: Self::NAME,
            version: Self::VERSION,
            interface: Self::INTERFACE,
        }
    }
}

// ===== Global =====

/// A runtime value global object.
#[derive(Debug)]
pub struct Global {
    pub name: &'static str,
    pub version: Version,
    pub interface: Interface,
}

impl Global {
    /// Create global from [`WlGlobal`] implementation.
    pub const fn of<G: WlGlobal>() -> Self {
        Self {
            name: G::NAME,
            version: G::VERSION,
            interface: G::INTERFACE,
        }
    }
}
