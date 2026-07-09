use todex::collections::slots::{Slots, IntoIter};
use todex::wayland::primitives::{AsObjectId, AsVersion, ObjectId, Version};
use todex::wayland::object::{AsNewId, NewId, Object, UnknownId, OccupiedNewId};
use todex::wayland::interface::{AsInterface, Interface};

use crate::handle::{Handle, WithHandle};

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
}

impl Objects {
    /// Returns true whether given id can be used in insertion.
    pub fn checks_id(&self, id: ObjectId) -> Result<(), OccupiedNewId> {
        let Some(idx) = id.to_u32().checked_sub(2) else {
            return Err(OccupiedNewId);
        };
        match self.slots.check_index(idx as usize) {
            true => Ok(()),
            false => Err(OccupiedNewId),
        }
    }

    /// Create new object from constructor message.
    ///
    /// # Panics.
    ///
    /// Panics if the new id cannot be used. Use [`Objects::checks_id`] to checks whether an id can
    /// be used in creation.
    pub fn create<M>(&mut self, msg: M) -> Object<M::Interface>
    where
        M: AsNewId<Interface: AsInterface> + AsVersion,
    {
        let new_id = msg.new_id();
        self.insert_inner(
            new_id.object_id(),
            ObjectEntry::from_parts(
                new_id.interface.interface(),
                msg.version(),
                Handle::from_idx(0),
            ),
        );
        Object::from_new_id(new_id)
    }

    /// Create new object from constructor message with a handle.
    ///
    /// # Panics.
    ///
    /// Panics if the new id cannot be used. Use [`Objects::checks_id`] to checks whether an id can
    /// be used in creation.
    pub fn create_with<M>(
        &mut self,
        msg: M,
        handle: Handle<<M::Interface as WithHandle>::Handle>,
    ) -> Object<M::Interface>
    where
        M: AsNewId<Interface: AsInterface> + AsVersion,
        M::Interface: WithHandle,
    {
        let new_id = msg.new_id();
        self.insert_inner(
            new_id.object_id(),
            ObjectEntry::from_parts(
                new_id.interface.interface(),
                msg.version(),
                handle.cast::<()>(),
            ),
        );
        Object::from_new_id(new_id)
    }

    /// Insert new object from parts.
    ///
    /// This is used by `wl_registry::bind` where the object type is a runtime value.
    ///
    /// # Panics.
    ///
    /// Panics if the new id cannot be used. Use [`Objects::checks_id`] to checks whether an id can
    /// be used in creation.
    pub fn insert_parts(&mut self, new_id: ObjectId, interface: Interface, version: Version) {
        self.insert_inner(
            new_id.object_id(),
            ObjectEntry::from_parts(interface, version, Handle::from_idx(0)),
        )
    }

    fn insert_inner(&mut self, object_id: ObjectId, object: ObjectEntry) {
        let idx = object_id
            .to_u32()
            .checked_sub(2)
            .unwrap_or_else(|| unchecked_new_id());
        self.slots
            .insert(idx as usize, object)
            .unwrap_or_else(|_| unchecked_new_id());
    }
}

impl Objects {
    /// This has the same effect of inserting the id and immediately remove it.
    pub fn use_one<I>(&mut self, new_id: NewId<I>) -> Object<I> {
        if let Some(idx) = new_id.object_id().to_u32().checked_sub(2) {
            self.slots.use_one(idx as usize);
        }
        Object::from_new_id(new_id)
    }

    /// Performs an object lookup.
    pub fn get(&self, id: ObjectId) -> Result<ObjectEntry, UnknownId> {
        let Some(idx) = id.to_u32().checked_sub(2) else {
            return Ok(wl_display());
        };
        self.slots.get(idx as usize).copied().ok_or(UnknownId)
    }

    /// Performs an object lookup.
    pub fn get_with<I>(&mut self, id: I) -> Result<ObjectEntry<Interface, I::Handle>, UnknownId>
    where
        I: AsObjectId + WithHandle,
    {
        let Some(idx) = id.object_id().to_u32().checked_sub(2) else {
            return Ok(wl_display());
        };
        self.slots
            .get(idx as usize)
            .copied()
            // this casting is fine, because its based on static type definition
            .map(|o| o.map_id(Handle::cast))
            .ok_or(UnknownId)
    }

    pub fn remove<O: AsObjectId>(&mut self, index: O) -> Result<ObjectEntry, UnknownId> {
        index
            .object_id()
            .to_u32()
            .checked_sub(2)
            .and_then(|i| self.slots.remove(i as usize))
            .ok_or(UnknownId)
    }
}

fn wl_display<H>() -> Object<Interface, Version, Handle<H>> {
    ObjectEntry::from_parts(Interface::WlDisplay, Version::ONE, Handle::<H>::from_idx(0))
}

/// New id checks can be automatically added before the handler, thus can make the insert operation
/// infallible by panicking, reducing error cases in handler.
#[cold]
#[inline(never)]
fn unchecked_new_id() -> ! {
    panic!("internal error: unchecked new id")
}

// ===== IntoIterator =====

impl IntoIterator for Objects {
    type Item = ObjectEntry;

    type IntoIter = IntoIter<ObjectEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.slots.into_iter()
    }
}
