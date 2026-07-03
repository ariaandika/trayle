#![expect(unused_imports)]
pub use buffer::{Buffers, Buffer};
pub use shm::{ShmPools, ShmPool};

pub use surface::{Surface, RoleOverwrite};
pub use surfaces::Surfaces;
pub use xdg_surface::XdgSurface;
pub use xdg_surfaces::XdgSurfaces;


mod buffer;
mod shm;

mod surface;
mod surfaces;

mod xdg_surface;
mod xdg_surfaces;
