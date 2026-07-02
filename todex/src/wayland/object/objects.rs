use crate::collections::slots::Slots;
use crate::wayland::primitives::{AsObjectId, AsVersion, ObjectId, Version};
use crate::wayland::object::{AsHandle, AsNewId, Handle, NewId, Object, ObjectError};
use crate::wayland::interface::{AsInterface, Interface};

use ObjectError as E;

pub type ObjectEntry = Object<Interface, Version, Handle>;

const INITIAL_CAP: usize = 32;

/// A list of wayland objects.
///
/// This is an array type where client can associate an index with given object.
///
/// Client can only append one index after the last used object slot. An attempt to insert past it
/// will result in an error.
pub struct Objects {
    slots: Slots<ObjectEntry>,
}

impl Objects {
    #[inline]
    pub fn new() -> Self {
        Self {
            slots: Slots::with_capacity(INITIAL_CAP),
        }
    }

    /// Create new object from constructor message.
    pub fn create<M>(&mut self, msg: M) -> Result<Object<M::Interface>, ObjectError>
    where
        M: AsNewId<Interface: AsInterface> + AsVersion,
    {
        self.create_handle(msg, Handle::default())
    }

    /// Create new object from constructor message.
    pub fn create_handle<M, H>(
        &mut self,
        msg: M,
        handle: H,
    ) -> Result<Object<M::Interface>, ObjectError>
    where
        M: AsNewId<Interface: AsInterface> + AsVersion,
        H: AsHandle,
    {
        let new_id = msg.new_id();
        self.insert_inner(
            new_id.object_id(),
            new_id.interface.interface(),
            msg.version(),
            handle.to_handle(),
        )?;
        Ok(Object::from_new_id(new_id))
    }

    /// Insert new object from parts.
    ///
    /// This is used by `wl_registry::bind` where the object type is a runtime value.
    pub fn insert_parts<I, H: AsHandle>(
        &mut self,
        new_id: NewId<I>,
        interface: Interface,
        version: Version,
        handle: H,
    ) -> Result<(), ObjectError> {
        self.insert_inner(new_id.object_id(), interface, version, handle.to_handle())
    }

    // detach the generics
    fn insert_inner(
        &mut self,
        object_id: ObjectId,
        interface: Interface,
        version: Version,
        handle: Handle,
    ) -> Result<(), ObjectError> {
        let entry = ObjectEntry::from_parts(interface, version, handle);
        let Some(idx) = object_id.to_u32().checked_sub(2).map(|e| e as usize) else {
            return Err(E::InvalidNewId);
        };
        match self.slots.insert(idx, entry) {
            Ok(()) => Ok(()),
            // NOTE: can be out of bounds error
            Err(_) => Err(E::OccupiedNewId),
        }
    }

    /// This has the same effect of inserting the id and immediately remove it.
    #[inline]
    pub fn use_one<I>(&mut self, new_id: NewId<I>) -> Object<I> {
        if let Some(idx) = new_id.object_id().to_u32().checked_sub(2) {
            self.slots.use_one(idx as usize);
        }
        Object::from_new_id(new_id)
    }

    /// Performs an object lookup.
    pub fn get_anon(&mut self, id: ObjectId) -> Result<ObjectEntry, ObjectError> {
        self.entry(id)
    }

    /// Performs an object lookup.
    pub fn get_mut<I: ObjectIndex>(&mut self, idx: I) -> Result<ObjectEntry, ObjectError> {
        ObjectIndex::get_object_mut(idx, self)
    }

    fn entry(&self, id: ObjectId) -> Result<ObjectEntry, ObjectError> {
        let Some(idx) = id.to_u32().checked_sub(2) else {
            return Ok(WL_DISPLAY);
        };
        self.slots.get(idx as usize).copied().ok_or(E::UnknownId)
    }
}

const WL_DISPLAY: ObjectEntry =
    ObjectEntry::from_parts(Interface::WlDisplay, Version::ONE, Handle::from_raw(0));

pub trait ObjectIndex {
    fn get_object_mut(self, objects: &mut Objects) -> Result<ObjectEntry, ObjectError>;
}

impl<I: AsInterface, M, D: AsObjectId> ObjectIndex for &Object<I, M, D> {
    #[inline]
    fn get_object_mut(self, objects: &mut Objects) -> Result<ObjectEntry, ObjectError> {
        let object = objects.entry(self.object_id())?;
        if object.interface() == self.interface() {
            Ok(object)
        } else {
            Err(E::InvalidId)
        }
    }
}

impl<I: AsInterface, M, D: AsObjectId> ObjectIndex for Object<I, M, D> {
    #[inline]
    fn get_object_mut(self, objects: &mut Objects) -> Result<ObjectEntry, ObjectError> {
        <&Self>::get_object_mut(&self, objects)
    }
}
