use crate::collections::slots::Slots;
use crate::wayland::handle::{AsHandle, Handle};
use crate::wayland::primitives::{AsObjectId, ObjectId, Version};
use crate::wayland::{AsInterface, Constructor, Interface, Object, ObjectError, WlObject};

use ObjectError as E;

type ObjectEntry = Object<Interface, Version, Handle>;

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
    pub fn create<C>(&mut self, constructor: C) -> Result<C::Interface, ObjectError>
    where
        C: Constructor,
        C::Interface: WlObject,
    {
        let object = constructor.new_id().create();
        self.insert_parts(
            object.object_id(),
            object.interface(),
            constructor.new_version(),
            Handle::default(),
        )?;
        Ok(object)
    }

    /// Create new object from constructor message.
    pub fn create_handle<C, H>(
        &mut self,
        constructor: C,
        handle: H,
    ) -> Result<C::Interface, ObjectError>
    where
        C: Constructor,
        C::Interface: WlObject,
        H: AsHandle,
    {
        let object = constructor.new_id().create();
        self.insert_parts(
            object.object_id(),
            object.interface(),
            constructor.new_version(),
            handle,
        )?;
        Ok(object)
    }

    /// Insert new object from parts.
    ///
    /// This is used by `wl_registry::bind` where the object type is a runtime value.
    pub fn insert_parts<H: AsHandle>(
        &mut self,
        object_id: ObjectId,
        interface: Interface,
        version: Version,
        handle: H,
    ) -> Result<(), ObjectError> {
        self.insert_inner(
            object_id,
            ObjectEntry::from_parts(interface, version, handle.to_handle()),
        )
    }

    fn insert_inner(&mut self, id: ObjectId, entry: ObjectEntry) -> Result<(), ObjectError> {
        let Some(idx) = id.to_u32().checked_sub(2).map(|e|e as usize) else {
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
    pub fn use_one<O: WlObject>(&mut self, object: &O) {
        let Some(idx) = object.object_id().to_u32().checked_sub(2) else {
            return;
        };
        self.slots.use_one(idx as usize);
    }

    /// Performs an object lookup.
    ///
    /// The index can be an [`ObjectId`], and returns the object [`Interface`] and associated data.
    /// If object id is `1`, returns [`Interface::WlDisplay`].
    ///
    /// Otherwise, [`Object`] can be used, and returns the associated object data. It also validate
    /// whether the interface is equal. Returns `None` if object id is `1`.
    ///
    /// Object data usually is an index referencing other resource. Object data are provided in
    /// object insertion.
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

impl ObjectIndex for ObjectId {
    #[inline]
    fn get_object_mut(self, objects: &mut Objects) -> Result<ObjectEntry, ObjectError> {
        objects.entry(self)
    }
}

impl<I: WlObject> ObjectIndex for &Object<I> {
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

impl<I: WlObject> ObjectIndex for Object<I> {
    #[inline]
    fn get_object_mut(self, objects: &mut Objects) -> Result<ObjectEntry, ObjectError> {
        <&Self>::get_object_mut(&self, objects)
    }
}
