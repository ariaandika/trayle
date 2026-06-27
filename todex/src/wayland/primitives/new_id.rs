use std::fmt;

use crate::wayland::primitives::{AsObjectId, ObjectId};

// ===== traits =====

/// Type that is associated with a new id.
pub trait AsNewId {
    type Interface;

    /// Returns the new id.
    fn new_id(&self) -> NewId<Self::Interface>;
}

// ===== NewId =====

/// A new id for an object.
///
/// Create the actual object using [`NewId::create`].
#[derive(Clone, Copy)]
pub struct NewId<I> {
    pub id: ObjectId,
    pub interface: I,
}

impl<I> NewId<I> {
    /// Create new `NewId`.
    #[inline]
    pub const fn new(id: ObjectId, interface: I) -> Self {
        Self { id, interface }
    }
}

impl<T> AsObjectId for NewId<T> {
    #[inline]
    fn object_id(&self) -> ObjectId {
        self.id
    }
}

impl<I: Clone> AsNewId for NewId<I> {
    type Interface = I;

    #[inline]
    fn new_id(&self) -> NewId<Self::Interface> {
        NewId::new(self.id, self.interface.clone())
    }
}

impl<T> fmt::Debug for NewId<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.id.fmt(f)
    }
}

impl<T> fmt::Display for NewId<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.id.fmt(f)
    }
}
