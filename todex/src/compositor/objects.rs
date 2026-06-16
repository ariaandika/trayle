use crate::collections::slots::Slots;
use crate::wayland::{AsInterface, AsObjectId, Interface, NewId};
use crate::wayland::{Object, ObjectError, ObjectId, WlObject};

use ObjectError as E;

const INITIAL_CAP: usize = 32;

/// A list of wayland objects.
///
/// This is an array type where client can associate an index with given object.
///
/// Client can only append one index after the last used object slot. An attempt to insert past it
/// will result in an error.
pub struct Objects {
    slots: Slots<(Interface, usize)>,
}

impl Objects {
    #[inline]
    pub fn new() -> Self {
        Self {
            slots: Slots::with_capacity(INITIAL_CAP),
        }
    }

    /// Create and insert new object from [`NewId`].
    #[inline]
    pub fn create<O: WlObject>(&mut self, new_id: NewId<O>) -> Result<O, ObjectError> {
        let object = new_id.create();
        self.insert_with(&object, 0)?;
        Ok(object)
    }

    /// Insert new object.
    #[inline]
    pub fn insert<O: WlObject>(&mut self, object: &O) -> Result<(), ObjectError> {
        self.insert_with(object, 0)
    }

    /// Insert new object with a value.
    ///
    /// The value can be retrieved in the [`Object`] struct.
    #[inline]
    pub fn insert_with<O: WlObject>(&mut self, object: &O, value: usize) -> Result<(), ObjectError> {
        self.insert_inner(object.object_id(), (object.interface(), value))
    }

    /// Insert new object from parts.
    ///
    /// This is used by `wl_registry::bind` where the object type is a runtime value.
    #[inline]
    pub fn insert_parts(&mut self, object_id: ObjectId, interface: Interface, value: usize) -> Result<(), ObjectError> {
        self.insert_inner(object_id, (interface, value))
    }

    fn insert_inner(&mut self, id: ObjectId, object: (Interface, usize)) -> Result<(), ObjectError> {
        let Some(idx) = id.to_u32().checked_sub(2).map(|e|e as usize) else {
            return Err(E::InvalidNewId);
        };
        match self.slots.insert(idx, object) {
            Ok(()) => Ok(()),
            // NOTE: can be out of bounds error
            Err(_) => Err(E::OccupiedNewId),
        }
    }

    /// This has the same effect of inserting the id and immediately remove it.
    #[inline]
    pub fn use_one<O: WlObject>(&mut self, object: &O) {
        let Some(idx) = object.object_id().to_u32().checked_sub(2).map(|e|e as usize) else {
            return;
        };
        self.slots.use_one(idx);
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
    pub fn get_mut<I: ObjectIndex>(&mut self, idx: I) -> Result<I::Output, ObjectError> {
        ObjectIndex::get_object_mut(idx, self)
    }

    fn entry(&self, id: ObjectId) -> Result<&(Interface, usize), ObjectError> {
        let Some(idx) = id.to_u32().checked_sub(2) else {
            return Ok(&(Interface::WlDisplay, 0));
        };
        self.slots.get(idx as usize).ok_or(E::UnknownId)
    }
}

pub trait ObjectIndex {
    type Output;

    fn get_object_mut(self, objects: &mut Objects) -> Result<Self::Output, ObjectError>;
}

impl ObjectIndex for ObjectId {
    type Output = Interface;

    #[inline]
    fn get_object_mut(self, objects: &mut Objects) -> Result<Self::Output, ObjectError> {
        objects.entry(self).map(|e| e.0)
    }
}

impl<I: WlObject> ObjectIndex for Object<I> {
    type Output = usize;

    #[inline]
    fn get_object_mut(self, objects: &mut Objects) -> Result<Self::Output, ObjectError> {
        let object = objects.entry(self.object_id())?;
        if object.0 == self.interface() {
            Ok(object.1)
        } else {
            Err(E::InvalidId)
        }
    }
}
