use todex::collections::slab::Slab;
use todex::wayland::object::{Handle, ObjectError};
use todex::wayland::error::WlError;

use crate::surface::XdgSurface;

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

    pub fn create(&mut self, surface_handle: Handle) -> Handle {
        let (idx, _) = self.buf.insert(XdgSurface::new(surface_handle));
        Handle::from_idx(idx)
    }

    pub fn get_mut(&mut self, handle: Handle) -> Result<&mut XdgSurface, WlError> {
        self.buf
            .get_mut(handle.to_idx())
            .ok_or(ObjectError::UnknownId.into())
    }

    pub fn remove(&mut self, handle: Handle) -> Result<XdgSurface, WlError> {
        self.buf
            .remove(handle.to_idx())
            .ok_or(ObjectError::UnknownId.into())
    }
}
