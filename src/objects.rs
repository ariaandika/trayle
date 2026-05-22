use std::ptr::NonNull;

use crate::alloc;
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
    ptr: NonNull<Entry>,
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
        const CAP: u32 = 32;
        Self {
            ptr: alloc::allocate(CAP),
            id: 0,
            len: 0,
            cap: CAP,
            last_delete: 0,
        }
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Object> {
        debug_assert!(!id.is_display());
        let idx = id.to_u32() - 2;
        if idx < self.len {
            match unsafe { self.ptr.add(idx as usize).as_mut() } {
                Entry::Some(object) => Some(object),
                Entry::None(_) => None,
            }
        } else {
            None
        }
    }
}

