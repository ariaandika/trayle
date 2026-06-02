use std::ptr::NonNull;

use crate::alloc;
use crate::wayland::{Interface, ObjectId, WaylandObject, WlError};

// ===== Object =====

pub struct Object {
    interface: Interface,
    value: usize,
}

impl Object {
    #[inline]
    pub fn interface(&self) -> Interface {
        self.interface
    }

    #[inline]
    pub fn value(&self) -> usize {
        self.value
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
    #[inline]
    pub fn new() -> Self {
        Self {
            ptr: alloc::allocate(INITIAL_CAP),
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

    /// Insert new object.
    #[inline]
    pub fn insert<O: WaylandObject>(&mut self, object: &O, value: usize) -> Result<(), WlError> {
        self.insert_inner(
            object.id(),
            Some(Object {
                interface: O::INTERFACE_ID,
                value,
            }),
        )
    }

    /// Insert new object from parts.
    ///
    /// This is used by `wl_registry::bind` where the object type is a runtime value.
    #[inline]
    pub fn insert_with(&mut self, object_id: ObjectId, interface: Interface, value: usize) -> Result<(), WlError> {
        self.insert_inner(object_id, Some(Object { interface, value }))
    }

    /// This has the same effect of inserting the id and immediately remove it.
    #[inline]
    pub fn use_one<O: WaylandObject>(&mut self, object: &O) -> Result<(), WlError> {
        self.insert_inner(object.id(), None)
    }

    fn insert_inner(&mut self, id: ObjectId, object: Option<Object>) -> Result<(), WlError> {
        if self.len == self.cap {
            self.grow();
        }

        debug_assert!(!id.is_display());
        let idx = (id.to_u32() - 2) as usize;

        // appending can only be one index after the last used id
        //
        // if it skips unused id, it will left skipped id unitialized
        if idx > self.len {
            return Err(WlError::InvalidNewId);
        }

        if idx == self.len {
            // SAFETY: `idx == len`
            unsafe { self.ptr.add(idx).write(object) };
            self.len += 1;
        } else {
            // SAFETY: `idx < len`
            let entry_mut = unsafe { self.ptr.add(idx).replace(object) };
            if entry_mut.is_some() {
                return Err(WlError::InvalidNewId);
            }
        }

        Ok(())
    }

    #[inline]
    pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut Object> {
        debug_assert!(!id.is_display());
        let idx = (id.to_u32() - 2) as usize;
        if idx < self.len {
            unsafe { self.ptr.add(idx).as_mut().as_mut() }
        } else {
            None
        }
    }
}

