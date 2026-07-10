use todex::collections::slab::Slab;

use crate::handle::Handle;
use crate::surface::XdgSurface;

const INITIAL_CAP: usize = 8;

pub struct XdgSurfaces {
    surfaces: Slab<XdgSurface>,
}

impl XdgSurfaces {
    pub fn new() -> Self {
        Self {
            surfaces: Slab::with_capacity(INITIAL_CAP),
        }
    }

    pub fn create(&mut self, xdg_surface: XdgSurface) -> Handle<XdgSurface> {
        Handle::from_idx(self.surfaces.insert(xdg_surface).0)
    }

    pub fn remove(&mut self, handle: Handle<XdgSurface>) -> XdgSurface {
        self.surfaces
            .remove(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}

impl std::ops::Index<Handle<XdgSurface>> for XdgSurfaces {
    type Output = XdgSurface;

    fn index(&self, handle: Handle<XdgSurface>) -> &Self::Output {
        self.surfaces
            .get(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}

impl std::ops::IndexMut<Handle<XdgSurface>> for XdgSurfaces {
    fn index_mut(&mut self, handle: Handle<XdgSurface>) -> &mut Self::Output {
        self.surfaces
            .get_mut(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}
