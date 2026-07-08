use todex::collections::slots::{Slots, IntoIter};
use todex::wayland::primitives::{AsObjectId, AsVersion, ObjectId, Version};
use todex::wayland::object::{AsNewId, NewId, Object, ObjectError};
use todex::wayland::interface::{AsInterface, Interface};

use crate::handle::{Handle, WithHandle};

use ObjectError as E;

pub type ObjectEntry<I = Interface, H = ()> = Object<I, Version, Handle<H>>;

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
    pub fn new() -> Self {
        Self {
            slots: Slots::with_capacity(INITIAL_CAP),
        }
    }
    /// Returns true whether given id can be used in insertion.
    pub fn checks_id(&self, id: ObjectId) -> Result<(), ObjectError> {
        let Some(idx) = id.to_u32().checked_sub(2) else {
            return Err(E::InvalidNewId);
        };
        match self.slots.check_index(idx as usize) {
            true => Ok(()),
            false => Err(E::OccupiedNewId),
        }
    }

    /// Create new object from constructor message.
    pub fn create<M>(&mut self, msg: M) -> Result<Object<M::Interface>, ObjectError>
    where
        M: AsNewId<Interface: AsInterface> + AsVersion,
    {
        let new_id = msg.new_id();
        self.insert_inner(
            new_id.object_id(),
            new_id.interface.interface(),
            msg.version(),
            Handle::from_idx(0),
        )?;
        Ok(Object::from_new_id(new_id))
    }

    /// Create new object from constructor message with a handle.
    pub fn create_with<M>(
        &mut self,
        msg: M,
        handle: Handle<<M::Interface as WithHandle>::Handle>,
    ) -> Result<Object<M::Interface>, ObjectError>
    where
        M: AsNewId<Interface: AsInterface> + AsVersion,
        M::Interface: WithHandle,
    {
        let new_id = msg.new_id();
        self.insert_inner(
            new_id.object_id(),
            new_id.interface.interface(),
            msg.version(),
            handle.cast::<()>(),
        )?;
        Ok(Object::from_new_id(new_id))
    }

    /// Insert new object from parts.
    ///
    /// This is used by `wl_registry::bind` where the object type is a runtime value.
    pub fn insert_parts(
        &mut self,
        new_id: ObjectId,
        interface: Interface,
        version: Version,
    ) -> Result<(), ObjectError> {
        self.insert_inner(new_id.object_id(), interface, version, Handle::from_idx(0))
    }

    // detach the generics
    fn insert_inner(
        &mut self,
        object_id: ObjectId,
        interface: Interface,
        version: Version,
        handle: Handle<()>,
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
    pub fn get_anon(&self, id: ObjectId) -> Result<ObjectEntry, ObjectError> {
        let Some(idx) = id.to_u32().checked_sub(2) else {
            return Ok(WL_DISPLAY);
        };
        self.slots.get(idx as usize).copied().ok_or(E::UnknownId)
    }

    /// Performs an object lookup.
    pub fn get_mut<I: ObjectIndex>(&mut self, idx: I) -> Result<ObjectEntry, ObjectError> {
        ObjectIndex::get_object_mut(idx, self)
    }

    /// Performs an object lookup.
    pub fn get_with<I: AsObjectId + WithHandle>(
        &mut self,
        id: I,
    ) -> Result<ObjectEntry<Interface, I::Handle>, ObjectError> {
        let Some(idx) = id.object_id().to_u32().checked_sub(2) else {
            return Ok(wl_display());
        };
        self.slots
            .get(idx as usize)
            .copied()
            // this casting is fine, because its based on static type definition
            .map(|o| o.map_id(Handle::cast))
            .ok_or(E::UnknownId)
    }

    #[inline]
    pub fn remove<O: AsObjectId>(&mut self, index: O) -> Result<ObjectEntry, ObjectError> {
        index
            .object_id()
            .to_u32()
            .checked_sub(2)
            .and_then(|i| self.slots.remove(i as usize))
            .ok_or(E::UnknownId)
    }
}

const WL_DISPLAY: ObjectEntry =
    ObjectEntry::from_parts(Interface::WlDisplay, Version::ONE, Handle::from_idx(0));

fn wl_display<H>() -> Object<Interface, Version, Handle<H>> {
    ObjectEntry::from_parts(Interface::WlDisplay, Version::ONE, Handle::<H>::from_idx(0))
}

pub trait ObjectIndex {
    fn get_object_mut(self, objects: &mut Objects) -> Result<ObjectEntry, ObjectError>;
}

impl<I: AsInterface, M, D: AsObjectId> ObjectIndex for &Object<I, M, D> {
    #[inline]
    fn get_object_mut(self, objects: &mut Objects) -> Result<ObjectEntry, ObjectError> {
        let object = objects.get_anon(self.object_id())?;
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

// ===== IntoIterator =====

impl IntoIterator for Objects {
    type Item = ObjectEntry;

    type IntoIter = IntoIter<ObjectEntry>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.slots.into_iter()
    }
}
