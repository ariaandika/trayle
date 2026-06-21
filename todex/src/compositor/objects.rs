use crate::collections::slots::Slots;
use crate::wayland::{AsInterface, AsObjectId, Constructor, Interface, NewId, ObjectData};
use crate::wayland::{Object, ObjectError, ObjectId, Version, WlObject};

use ObjectError as E;

const INITIAL_CAP: usize = 32;

#[derive(Debug, Copy, Clone)]
pub struct ObjectEntry {
    interface: Interface,
    version: Version,
    data: u32,
}

impl AsInterface for ObjectEntry {
    fn interface(&self) -> Interface {
        self.interface
    }
}

impl ObjectEntry {
    const WL_DISPLAY: Self = Self {
        interface: Interface::WlDisplay,
        version: Version::new(1).unwrap(),
        data: 0,
    };

    #[inline]
    pub fn version(&self) -> Version {
        self.version
    }

    pub fn data<D: ObjectData>(&self) -> D {
        D::from_raw(self.data)
    }
}

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

    /// Create and insert new object from [`NewId`].
    pub fn create<O: WlObject>(&mut self, new_id: NewId<O>) -> Result<O, ObjectError> {
        let object = new_id.create();
        self.insert_parts(object.object_id(), object.interface(), Version::ONE, ())?;
        Ok(object)
    }

    /// Create and insert new object from [`NewId`].
    pub fn create2<C>(&mut self, constructor: C) -> Result<C::Interface, ObjectError>
    where
        C: Constructor,
        C::Interface: WlObject,
    {
        let new_id = constructor.new_id();
        let object = new_id.create();
        let version = constructor.new_version();
        self.insert_parts(object.object_id(), object.interface(), version, ())?;
        Ok(object)
    }

    /// Insert new object.
    pub fn insert<O: WlObject>(&mut self, object: &O) -> Result<(), ObjectError> {
        self.insert_with(object, 0)
    }

    /// Insert new object with a data.
    ///
    /// The data can be retrieved in lookup operation.
    pub fn insert_with<O: WlObject>(&mut self, object: &O, data: u32) -> Result<(), ObjectError> {
        self.insert_inner(
            object.object_id(),
            ObjectEntry {
                interface: object.interface(),
                version: Version::ONE,
                data,
            },
        )
    }

    /// Insert new object from parts.
    ///
    /// This is used by `wl_registry::bind` where the object type is a runtime value.
    pub fn insert_parts<D: ObjectData>(
        &mut self,
        object_id: ObjectId,
        interface: Interface,
        version: Version,
        data: D,
    ) -> Result<(), ObjectError> {
        self.insert_inner(
            object_id,
            ObjectEntry {
                interface,
                version,
                data: data.to_raw(),
            },
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
            return Ok(ObjectEntry::WL_DISPLAY);
        };
        self.slots.get(idx as usize).copied().ok_or(E::UnknownId)
    }
}

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
        if object.interface == self.interface() {
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
