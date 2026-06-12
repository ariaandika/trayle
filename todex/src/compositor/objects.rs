use std::ptr::NonNull;

use crate::alloc;
use crate::wayland::{AsObjectId, Interface, NewId, Object, ObjectId, WlError, WlObject};

const INITIAL_CAP: usize = 32;

/// A list of wayland objects.
///
/// This is a list where each object is an `Option`. Client can append or reuse removed object slot.
///
/// Client can only append one index after the last used object slot. An attempt to insert past it
/// will result in an error.
pub struct Objects {
    ptr: NonNull<Option<(Interface, usize)>>,
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

    /// Create and insert new object from [`NewId`].
    #[inline]
    pub fn create<O: WlObject>(&mut self, new_id: NewId<O>) -> Result<O, WlError> {
        let object = new_id.create();
        self.insert_with(&object, 0)?;
        Ok(object)
    }

    /// Insert new object.
    #[inline]
    pub fn insert<O: WlObject>(&mut self, object: &O) -> Result<(), WlError> {
        self.insert_with(object, 0)
    }

    /// Insert new object with a value.
    ///
    /// The value can be retrieved in the [`Object`] struct.
    #[inline]
    pub fn insert_with<O: WlObject>(&mut self, object: &O, value: usize) -> Result<(), WlError> {
        self.insert_inner(object.object_id(), Some((O::INTERFACE, value)))
    }

    /// Insert new object from parts.
    ///
    /// This is used by `wl_registry::bind` where the object type is a runtime value.
    #[inline]
    pub fn insert_parts(&mut self, object_id: ObjectId, interface: Interface, value: usize) -> Result<(), WlError> {
        self.insert_inner(object_id, Some((interface, value)))
    }

    /// This has the same effect of inserting the id and immediately remove it.
    #[inline]
    pub fn use_one<O: WlObject>(&mut self, object: &O) -> Result<(), WlError> {
        self.insert_inner(object.object_id(), None)
    }

    fn insert_inner(&mut self, id: ObjectId, object: Option<(Interface, usize)>) -> Result<(), WlError> {
        if self.len == self.cap {
            self.grow();
        }

        let Some(idx) = id.to_u32().checked_sub(2).map(|e|e as usize) else {
            return Err(WlError::InvalidNewId);
        };

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

    /// Performs an object lookup.
    ///
    /// The index can be an [`ObjectId`], and returns the object [`Interface`]. If object id is `1`,
    /// returns [`Interface::WlDisplay`].
    ///
    /// Otherwise, [`Object`] can be used, and returns the associated object value. It also validate
    /// whether the interface is equal. Returns `None` if object id is `1`.
    ///
    /// Object value usually is an index referencing other resource. Object value are provided in
    /// object insertion.
    pub fn get_mut<I: ObjectIndex>(&mut self, idx: I) -> Option<I::Output> {
        ObjectIndex::get_object_mut(idx, self)
    }
}

pub trait ObjectIndex {
    type Output;

    fn get_object_mut(self, objects: &mut Objects) -> Option<Self::Output>;
}

impl ObjectIndex for ObjectId {
    type Output = Interface;

    #[inline]
    fn get_object_mut(self, objects: &mut Objects) -> Option<Self::Output> {
        let Some(idx) = self.to_u32().checked_sub(2) else {
            return Some(Interface::WlDisplay)
        };
        if (idx as usize) < objects.len {
            unsafe { objects.ptr.add(idx as usize).as_mut() }.map(|e| e.0)
        } else {
            None
        }
    }
}

impl<I: WlObject> ObjectIndex for Object<I> {
    type Output = usize;

    #[inline]
    fn get_object_mut(self, objects: &mut Objects) -> Option<Self::Output> {
        let idx = self.object_id().to_u32().checked_sub(2)? as usize;
        if idx >= objects.len {
            return None;
        }
        let object = unsafe { objects.ptr.add(idx).as_ref() }.as_ref()?;
        if object.0 == I::INTERFACE {
            Some(object.1)
        } else {
            None
        }
    }
}
