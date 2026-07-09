use todex::collections::slab::Slab;

use crate::handle::Handle;
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

    pub fn remove(&mut self, handle: Handle<Surface>) -> Surface {
        self.buf
            .remove(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}

impl std::ops::Index<Handle<Surface>> for Surfaces {
    type Output = Surface;

    fn index(&self, handle: Handle<Surface>) -> &Self::Output {
        self.buf
            .get(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}

impl std::ops::IndexMut<Handle<Surface>> for Surfaces {
    fn index_mut(&mut self, handle: Handle<Surface>) -> &mut Self::Output {
        self.buf
            .get_mut(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}
