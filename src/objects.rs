use std::ptr::NonNull;

use crate::alloc;
use crate::wayland::{Id, Interface, WlError, WlObject};

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
    ptr: NonNull<Option<Object>>,
    len: u32,
    cap: u32,
}

impl Drop for Objects {
    fn drop(&mut self) {
        alloc::deallocate(self.ptr, self.cap);
    }
}

impl Objects {
    pub fn new() -> Self {
        const CAP: u32 = 32;
        Self {
            ptr: alloc::allocate(CAP),
            len: 0,
            cap: CAP,
        }
    }

    pub fn insert<O: WlObject>(&mut self, object: &O) -> Result<(), WlError> {
        self.insert_inner(object.id(), O::INTERFACE)
    }

    fn insert_inner(&mut self, id: Id, interface: Interface) -> Result<(), WlError> {
        debug_assert!(!id.is_display());
        let idx = id.to_u32() - 2;

        // there will always be space after the last element, so appending is ok, but if it skips
        // unused id, it will left skipped id unitialized
        if idx > self.len {
            return Err(WlError::InvalidNewId);
        }

        let entry_mut = unsafe { self.ptr.add(idx as usize).as_mut() };
        if entry_mut.is_some() {
            return Err(WlError::InvalidNewId);
        }
        *entry_mut = Some(Object { interface });
        if idx == self.len {
            // append new entry
            self.len += 1;
        }

        // make sure there is available space after the last element
        if self.cap - self.len < 4 {
            self.cap = alloc::grow_exp(&mut self.ptr, self.cap);
        }

        Ok(())
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Object> {
        debug_assert!(!id.is_display());
        let idx = id.to_u32() - 2;
        if idx < self.len {
            unsafe { self.ptr.add(idx as usize).as_mut().as_mut() }
        } else {
            None
        }
    }
}

