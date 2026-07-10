use todex::collections::slab::Slab;
use todex::wayland::interface::wl_surface;

use crate::handle::Handle;
use crate::surface::Surface;

const INITIAL_CAP: usize = 32;

pub struct Surfaces {
    surfaces: Slab<Surface>,
}

impl Surfaces {
    pub fn new() -> Self {
        Self {
            surfaces: Slab::with_capacity(INITIAL_CAP),
        }
    }

    /// Create new [`Surface`].
    pub fn create(&mut self) -> Handle<Surface> {
        let (idx, _) = self.surfaces.insert(Surface::new());
        Handle::from_idx(idx)
    }

    /// Remove and destroy [`Surface`].
    ///
    /// Role object must be destroyed before its surface.
    pub fn remove(&mut self, handle: Handle<Surface>) -> Result<Surface, wl_surface::Error> {
        let surface = &mut self[handle];
        if surface.has_role() {
            return Err(wl_surface::Error::DefunctRoleObject);
        }
        self.surfaces
            .remove(handle.to_idx())
            .map_or_else(|| handle.dangling(), Ok)
    }
}

impl std::ops::Index<Handle<Surface>> for Surfaces {
    type Output = Surface;

    fn index(&self, handle: Handle<Surface>) -> &Self::Output {
        self.surfaces
            .get(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}

impl std::ops::IndexMut<Handle<Surface>> for Surfaces {
    fn index_mut(&mut self, handle: Handle<Surface>) -> &mut Self::Output {
        self.surfaces
            .get_mut(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}
