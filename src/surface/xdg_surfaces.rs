use todex::collections::slab::Slab;
use todex::wayland::object::ObjectError;

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

    pub fn get_mut(&mut self, handle: Handle<XdgSurface>) -> Result<&mut XdgSurface, ObjectError> {
        self.buf
            .get_mut(handle.to_idx())
            .ok_or(ObjectError::UnknownId)
    }

    pub fn remove(&mut self, handle: Handle<XdgSurface>) -> Result<XdgSurface, ObjectError> {
        self.buf
            .remove(handle.to_idx())
            .ok_or(ObjectError::UnknownId)
    }
}
