//! Wayland surface.
//!
//! [`Surface`] is the core generic api.
//!
//! [`Surface`] can be assined a [`Role`].
pub use region::Region;
pub use regions::Regions;
pub use role::{Role, RoleError};
pub use surface::{Surface};
pub use surfaces::Surfaces;
pub use xdg_surface::XdgSurface;
pub use xdg_surfaces::XdgSurfaces;

// ===== core =====

mod region;
mod role;
mod surface;

// ===== roles =====

mod xdg_surface;

// ===== collections =====

mod regions;
mod surfaces;
mod xdg_surfaces;
