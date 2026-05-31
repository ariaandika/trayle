use std::ptr::NonNull;

use crate::collections::alloc;
use crate::wayland::{Id, InterfaceId, Object as WlObject, WlError};

// ===== Object =====

pub struct Object {
    interface: InterfaceId,
}

impl Object {
    pub fn interface(&self) -> InterfaceId {
        self.interface
    }
}

// ===== Objects =====

const INITIAL_CAP: usize = 32;

/// A list of wayland objects.
///
/// This is a list where each object is an `Option`. Client can append or reuse removed object slot.
///
/// Client can only append one index after the last used object slot. An attempt to insert past it
/// will result in an error.
pub struct Objects {
    /// the `len`-nth entry will always be initialized.
    ptr: NonNull<Option<Object>>,
    len: usize,
    cap: usize,
}

impl Drop for Objects {
    fn drop(&mut self) {
        // if in the future, `Object` contains something that needs `Drop`, this should be changed.
        alloc::deallocate(self.ptr);
    }
}

impl Objects {
    pub fn new() -> Self {
        let ptr = alloc::allocate(INITIAL_CAP);
        // initialize the `len`-nth entry
        unsafe { ptr.write(None) };
        Self {
            ptr,
            len: 0,
            cap: INITIAL_CAP,
        }
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self) {
        let new_cap = alloc::calc_exp(self.cap);
        self.ptr = alloc::reallocate(self.ptr, new_cap);
        self.cap = new_cap;
    }

    pub fn insert_object<O: WlObject>(&mut self, object: &O) -> Result<(), WlError> {
        self.insert_inner(
            object.id(),
            Some(Object {
                interface: O::INTERFACE_ID,
            }),
        )
    }

    pub fn insert(&mut self, id: Id, interface: InterfaceId) -> Result<(), WlError> {
        self.insert_inner(id, Some(Object { interface }))
    }

    /// This has the same effect of inserting the id and immediately remove it.
    pub fn use_one<O: WlObject>(&mut self, object: &O) -> Result<(), WlError> {
        self.insert_inner(object.id(), None)
    }

    fn insert_inner(&mut self, id: Id, object: Option<Object>) -> Result<(), WlError> {
        debug_assert!(!id.is_display());
        let idx = (id.to_u32() - 2) as usize;

        // appending can only be one index after the last used id
        //
        // if it skips unused id, it will left skipped id unitialized
        if idx > self.len {
            return Err(WlError::InvalidNewId);
        }

        // SAFETY: `idx <= len`, and the `len`-nth entry is always initialized
        let entry_mut = unsafe { self.ptr.add(idx).replace(object) };
        if entry_mut.is_some() {
            return Err(WlError::InvalidNewId);
        }
        if idx == self.len {
            // appending, increase the length
            self.len += 1;
            // initialize the `len`-nth entry
            unsafe { self.ptr.add(self.len).write(None) };
        }

        // make sure there is available space after the last element
        if self.cap - self.len < 4 {
            self.grow();
        }

        Ok(())
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Object> {
        debug_assert!(!id.is_display());
        let idx = (id.to_u32() - 2) as usize;
        if idx < self.len {
            unsafe { self.ptr.add(idx).as_mut().as_mut() }
        } else {
            None
        }
    }
}

