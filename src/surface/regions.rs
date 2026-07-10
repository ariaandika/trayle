use todex::collections::slab::Slab;

use crate::handle::Handle;
use crate::surface::Region;

const INITIAL_CAP: usize = 8;

pub struct Regions {
    regions: Slab<Region>,
}

impl Regions {
    pub fn new() -> Self {
        Self {
            regions: Slab::with_capacity(INITIAL_CAP),
        }
    }

    pub fn create(&mut self) -> Handle<Region> {
        Handle::from_idx(self.regions.insert(Region::new()).0)
    }

    pub fn remove(&mut self, handle: Handle<Region>) -> Region {
        self.regions
            .remove(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}

impl std::ops::Index<Handle<Region>> for Regions {
    type Output = Region;

    fn index(&self, handle: Handle<Region>) -> &Self::Output {
        self.regions
            .get(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}

impl std::ops::IndexMut<Handle<Region>> for Regions {
    fn index_mut(&mut self, handle: Handle<Region>) -> &mut Self::Output {
        self.regions
            .get_mut(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}
