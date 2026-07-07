use std::fmt;

use crate::wayland::primitives::{AsObjectId, ObjectId};
use crate::wayland::interface::WlInterface;

// ===== traits =====

/// Type that is associated with a new id.
pub trait AsNewId {
    type Interface;

    /// Returns the new id.
    fn new_id(&self) -> NewId<Self::Interface>;
}

impl<I: AsNewId> AsNewId for &I {
    type Interface = I::Interface;

    #[inline]
    fn new_id(&self) -> NewId<Self::Interface> {
        I::new_id(self)
    }
}

// ===== NewId =====

/// A new id for an object.
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

impl<I: WlInterface> AsNewId for NewId<I> {
    type Interface = I;

    #[inline]
    fn new_id(&self) -> NewId<Self::Interface> {
        *self
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
