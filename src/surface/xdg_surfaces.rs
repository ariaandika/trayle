use todex::collections::slab::Slab;

use crate::handle::Handle;
use crate::surface::{XdgSurface, Surface};

const INITIAL_CAP: usize = 16;

pub struct XdgSurfaces {
    buf: Slab<XdgSurface>,
}

impl XdgSurfaces {
    pub fn new() -> Self {
        Self {
            buf: Slab::with_capacity(INITIAL_CAP),
        }
    }

    pub fn create(&mut self, surface_handle: Handle<Surface>) -> Handle<XdgSurface> {
        let (idx, _) = self.buf.insert(XdgSurface::new(surface_handle));
        Handle::from_idx(idx)
    }

    pub fn remove(&mut self, handle: Handle<XdgSurface>) -> XdgSurface {
        self.buf
            .remove(handle.to_idx())
            .unwrap_or_else(||handle.dangling())
    }
}

impl std::ops::Index<Handle<XdgSurface>> for XdgSurfaces {
    type Output = XdgSurface;

    fn index(&self, handle: Handle<XdgSurface>) -> &Self::Output {
        self.buf
            .get(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}

impl std::ops::IndexMut<Handle<XdgSurface>> for XdgSurfaces {
    fn index_mut(&mut self, handle: Handle<XdgSurface>) -> &mut Self::Output {
        self.buf
            .get_mut(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}
