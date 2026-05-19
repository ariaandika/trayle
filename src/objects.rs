use crate::ptr::Ptr;
use crate::wayland::{Id, Interface};

// ===== Object =====

pub struct Object {
    interface: Interface,
}

impl Object {
    pub fn interface(&self) -> Interface {
        self.interface
    }
}

// ===== Objects =====

pub struct Objects {
    ptr: Ptr<Entry>,
    id: u32,
    len: u32,
    cap: u32,
    last_delete: u32,
}

enum Entry {
    Some(Object),
    None(u32)
}

impl Objects {
    pub fn new() -> Self {
        Self {
            ptr: Ptr::allocate(32),
            id: 0,
            len: 0,
            cap: 32,
            last_delete: 0,
        }
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Object> {
        debug_assert!(!id.is_display());
        let idx = id.as_u32() - 2;
        if idx < self.len {
            match self.ptr.add(idx).as_mut() {
                Entry::Some(object) => Some(object),
                Entry::None(_) => None,
            }
        } else {
            None
        }
    }
}

