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
        let ptr = alloc::allocate(CAP);
        // initialize the `len`-nth entry
        unsafe { ptr.write(None) };
        Self {
            ptr,
            len: 0,
            cap: CAP,
        }
    }

    pub fn insert_object<O: WlObject>(&mut self, object: &O) -> Result<(), WlError> {
        self.insert(object.id(), O::INTERFACE)
    }

    pub fn insert(&mut self, id: Id, interface: Interface) -> Result<(), WlError> {
        debug_assert!(!id.is_display());
        let idx = id.to_u32() - 2;

        // there will always be space after the last element, so appending is ok, but if it skips
        // unused id, it will left skipped id unitialized
        if idx > self.len {
            return Err(WlError::InvalidNewId);
        }

        // SAFETY: `idx <= len`, and the `len`-nth entry is always initialized
        let entry_mut = unsafe { self.ptr.add(idx as usize).replace(Some(Object { interface })) };
        if entry_mut.is_some() {
            return Err(WlError::InvalidNewId);
        }
        if idx == self.len {
            // append new entry
            self.len += 1;
            // initialize the `len`-nth entry
            unsafe { self.ptr.add(self.len as usize).write(None) };
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

