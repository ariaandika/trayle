use todex::collections::slab::Slab;
use todex::wayland::object::{Handle, ObjectError};
use todex::wayland::error::WlError;

use crate::wayland::Surface;

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

    pub fn create(&mut self) -> Handle {
        let (idx, _) = self.buf.insert(Surface::new());
        Handle::from_idx(idx)
    }

    pub fn get_mut(&mut self, handle: Handle) -> Result<&mut Surface, WlError> {
        self.buf
            .get_mut(handle.to_idx())
            .ok_or(ObjectError::UnknownId.into())
    }
}
