use todex::handle::Handle;
use todex::collections::slab::Slab;
use todex::wayland::object::ObjectError;

use crate::surface::Surface;

const INITIAL_CAP: usize = 32;

pub struct Surfaces {
    buf: Slab<Surface>,
}

impl Surfaces {
    pub fn new() -> Self {
        Self {
            buf: Slab::with_capacity(INITIAL_CAP),
        }
    }

    pub fn create(&mut self) -> Handle<Surface> {
        let (idx, _) = self.buf.insert(Surface::new());
        Handle::from_idx(idx)
    }

    pub fn get_mut(&mut self, handle: Handle<Surface>) -> Result<&mut Surface, ObjectError> {
        self.buf
            .get_mut(handle.to_idx())
            .ok_or(ObjectError::UnknownId)
    }
}
